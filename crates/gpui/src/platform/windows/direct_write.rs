use std::{
    borrow::{Borrow, Cow},
    mem::ManuallyDrop,
    sync::Arc,
};

use anyhow::{anyhow, Result};
use collections::HashMap;
use itertools::Itertools;
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use smallvec::SmallVec;
use util::ResultExt;
use windows::{
    core::{implement, HRESULT, HSTRING, PCWSTR},
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::{BOOL, COLORREF, DWRITE_E_NOCOLOR, RECT},
        Globalization::GetUserDefaultLocaleName,
        Graphics::{
            Direct2D::{
                Common::{
                    D2D1_ALPHA_MODE_IGNORE, D2D1_ALPHA_MODE_PREMULTIPLIED,
                    D2D1_ALPHA_MODE_STRAIGHT, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_POINT_2F,
                    D2D_SIZE_F, D2D_SIZE_U,
                },
                D2D1CreateFactory, ID2D1Bitmap1, ID2D1Factory, D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                D2D1_BITMAP_OPTIONS_CPU_READ, D2D1_BITMAP_PROPERTIES, D2D1_BITMAP_PROPERTIES1,
                D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS,
                D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS_GDI_COMPATIBLE,
                D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
                D2D1_MAP_OPTIONS_READ, D2D1_RENDER_TARGET_PROPERTIES,
                D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_TYPE_SOFTWARE,
                D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,
            },
            DirectWrite::*,
            Dxgi::Common::{DXGI_FORMAT_A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM},
            Gdi::{
                CreateBitmap, CreateCompatibleBitmap, CreateCompatibleDC, DeleteObject,
                GetCurrentObject, GetDIBits, GetObjectW, GetStockObject, Rectangle, ReleaseDC,
                SelectObject, SetBoundsRect, SetDCBrushColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
                DCB_ENABLE, DCB_RESET, DC_BRUSH, DIBSECTION, DIB_RGB_COLORS, HDC, NULL_PEN,
                OBJ_BITMAP, SET_BOUNDS_RECT_FLAGS,
            },
        },
    },
};

use crate::{
    point, px, Bounds, DevicePixels, Font, FontFeatures, FontId, FontMetrics, FontRun, FontStyle,
    FontWeight, GlyphId, LineLayout, Pixels, PlatformTextSystem, Point, RenderGlyphParams,
    ShapedGlyph, ShapedRun, Size, SUBPIXEL_VARIANTS,
};

struct FontInfo {
    font_family: String,
    font_face: IDWriteFontFace3,
    font_set_index: usize,
    features: Vec<DWRITE_FONT_FEATURE>,
    is_emoji: bool,
}

pub(crate) struct DirectWriteTextSystem(RwLock<DirectWriteState>);

struct DirectWriteComponent {
    locale: String,
    factory: IDWriteFactory5,
    in_memory_loader: IDWriteInMemoryFontFileLoader,
    builder: IDWriteFontSetBuilder1,
    gdi: IDWriteGdiInterop,
    render_target: IDWriteBitmapRenderTarget3,
}

struct DirectWriteState {
    components: DirectWriteComponent,
    font_sets: Vec<IDWriteFontSet>,
    fonts: Vec<FontInfo>,
    font_selections: HashMap<Font, FontId>,
    font_id_by_postscript_name: HashMap<String, FontId>,
}

impl DirectWriteComponent {
    pub fn new() -> Self {
        unsafe {
            let factory: IDWriteFactory5 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            let in_memory_loader = factory.CreateInMemoryFontFileLoader().unwrap();
            factory.RegisterFontFileLoader(&in_memory_loader).unwrap();
            let builder = factory.CreateFontSetBuilder2().unwrap();
            let mut locale_vec = vec![0u16; 512];
            GetUserDefaultLocaleName(&mut locale_vec);
            let locale = String::from_utf16_lossy(&locale_vec);
            let gdi = factory.GetGdiInterop().unwrap();
            let bitmap_render_target = gdi.CreateBitmapRenderTarget(None, 10, 10).unwrap();
            let render_target: IDWriteBitmapRenderTarget3 =
                std::mem::transmute(bitmap_render_target);

            DirectWriteComponent {
                locale,
                factory,
                in_memory_loader,
                builder,
                gdi,
                render_target,
            }
        }
    }
}

impl DirectWriteTextSystem {
    pub(crate) fn new() -> Self {
        let components = DirectWriteComponent::new();
        let system_set = unsafe { components.factory.GetSystemFontSet().unwrap() };

        Self(RwLock::new(DirectWriteState {
            components: DirectWriteComponent::new(),
            font_sets: vec![system_set],
            fonts: Vec::new(),
            font_selections: HashMap::default(),
            font_id_by_postscript_name: HashMap::default(),
        }))
    }
}

impl Default for DirectWriteTextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformTextSystem for DirectWriteTextSystem {
    fn add_fonts(&self, fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        self.0.write().add_fonts(fonts)
    }

    fn all_font_names(&self) -> Vec<String> {
        self.0.read().all_font_names()
    }

    fn all_font_families(&self) -> Vec<String> {
        self.0.read().all_font_families()
    }

    fn font_id(&self, font: &Font) -> Result<FontId> {
        let lock = self.0.upgradable_read();
        if let Some(font_id) = lock.font_selections.get(font) {
            Ok(*font_id)
        } else {
            let mut lock = RwLockUpgradableReadGuard::upgrade(lock);
            let font_id = lock.select_font(font).unwrap();
            lock.font_selections.insert(font.clone(), font_id);
            Ok(font_id)
        }
    }

    fn font_metrics(&self, font_id: FontId) -> FontMetrics {
        self.0.read().font_metrics(font_id)
    }

    fn typographic_bounds(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Bounds<f32>> {
        self.0.read().get_typographic_bounds(font_id, glyph_id)
    }

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> anyhow::Result<Size<f32>> {
        self.0.read().get_advance(font_id, glyph_id)
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        self.0.read().glyph_for_char(font_id, ch)
    }

    fn glyph_raster_bounds(
        &self,
        params: &RenderGlyphParams,
    ) -> anyhow::Result<Bounds<DevicePixels>> {
        self.0.read().raster_bounds(params)
    }

    fn rasterize_glyph(
        &self,
        params: &RenderGlyphParams,
        raster_bounds: Bounds<DevicePixels>,
    ) -> anyhow::Result<(Size<DevicePixels>, Vec<u8>)> {
        self.0.read().rasterize_glyph(params, raster_bounds)
    }

    fn layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> LineLayout {
        self.0.write().layout_line(text, font_size, runs)
    }

    fn wrap_line(
        &self,
        _text: &str,
        _font_id: FontId,
        _font_size: Pixels,
        _width: Pixels,
    ) -> Vec<usize> {
        // self.0.read().wrap_line(text, font_id, font_size, width)
        unimplemented!()
    }
}

impl DirectWriteState {
    fn add_fonts(&mut self, fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        for font_data in fonts {
            match font_data {
                Cow::Borrowed(data) => unsafe {
                    let font_file = self
                        .components
                        .in_memory_loader
                        .CreateInMemoryFontFileReference(
                            &self.components.factory,
                            data.as_ptr() as _,
                            data.len() as _,
                            None,
                        )?;
                    self.components.builder.AddFontFile(&font_file)?;
                },
                Cow::Owned(data) => unsafe {
                    let font_file = self
                        .components
                        .in_memory_loader
                        .CreateInMemoryFontFileReference(
                            &self.components.factory,
                            data.as_ptr() as _,
                            data.len() as _,
                            None,
                        )?;
                    self.components.builder.AddFontFile(&font_file)?;
                },
            }
        }
        let set = unsafe { self.components.builder.CreateFontSet()? };
        self.font_sets.push(set);

        Ok(())
    }

    fn select_font(&mut self, target_font: &Font) -> Option<FontId> {
        unsafe {
            for (fontset_index, fontset) in self.font_sets.iter().enumerate() {
                let font = fontset
                    .GetMatchingFonts(
                        &HSTRING::from(target_font.family.to_string()),
                        // DWRITE_FONT_WEIGHT(target_font.weight.0 as _),
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                    )
                    .unwrap();
                let total_number = font.GetFontCount();
                for _ in 0..total_number {
                    let font_face_ref = font.GetFontFaceReference(0).unwrap();
                    let Some(font_face) = font_face_ref.CreateFontFace().log_err() else {
                        continue;
                    };
                    let Some(postscript_name) = get_postscript_name(&font_face) else {
                        continue;
                    };
                    let font_family = target_font.family.to_string();
                    let is_emoji = font_face.IsColorFont().as_bool();
                    println!("post: {}, emoji: {}", postscript_name, is_emoji);
                    let font_info = FontInfo {
                        font_family,
                        font_face,
                        font_set_index: fontset_index,
                        features: direct_write_features(&target_font.features),
                        is_emoji,
                    };
                    let font_id = FontId(self.fonts.len());
                    self.fonts.push(font_info);
                    return Some(font_id);
                }
            }
            None
        }
    }

    fn select_font_by_family(&mut self, family: String) -> Option<FontId> {
        unsafe {
            for (fontset_index, fontset) in self.font_sets.iter().enumerate() {
                let font = fontset
                    .GetMatchingFonts(
                        &HSTRING::from(&family),
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                    )
                    .unwrap();
                let total_number = font.GetFontCount();
                for _ in 0..total_number {
                    let font_face_ref = font.GetFontFaceReference(0).unwrap();
                    let Some(font_face) = font_face_ref.CreateFontFace().log_err() else {
                        continue;
                    };
                    let Some(postscript_name) = get_postscript_name(&font_face) else {
                        continue;
                    };
                    let is_emoji = font_face.IsColorFont().as_bool();
                    println!("post: {}, emoji: {}", postscript_name, is_emoji);
                    let font_info = FontInfo {
                        font_family: family,
                        font_face,
                        font_set_index: fontset_index,
                        features: Vec::new(),
                        is_emoji,
                    };
                    let font_id = FontId(self.fonts.len());
                    self.fonts.push(font_info);
                    return Some(font_id);
                }
            }
            None
        }
    }

    // unsafe fn calculate_line_metrics(
    //     &mut self,
    //     index_start: &mut usize,
    //     ascent: &mut f32,
    //     descent: &mut f32,
    //     font_set_index: usize,
    //     font_family_name: String,
    //     font_size: f32,
    //     locale_name: PCWSTR,
    //     text_wide: &[u16],
    //     font_weight: DWRITE_FONT_WEIGHT,
    //     font_style: DWRITE_FONT_STYLE,
    // ) -> (f32, Vec<ShapedRun>) {
    //     let collection = {
    //         let font_set = &self.font_sets[font_set_index];
    //         self.components
    //             .factory
    //             .CreateFontCollectionFromFontSet(font_set)
    //             .unwrap()
    //     };
    //     let format = self
    //         .components
    //         .factory
    //         .CreateTextFormat(
    //             &HSTRING::from(&font_family_name),
    //             &collection,
    //             font_weight,
    //             font_style,
    //             DWRITE_FONT_STRETCH_NORMAL,
    //             font_size,
    //             locale_name,
    //         )
    //         .unwrap();
    //     let layout = self
    //         .components
    //         .factory
    //         .CreateTextLayout(text_wide, &format, f32::INFINITY, f32::INFINITY)
    //         .unwrap();

    //     let renderer_inner = Arc::new(RwLock::new(TextRendererInner::new()));
    //     let renderer: IDWriteTextRenderer =
    //         TextRenderer::new(renderer_inner.clone(), locale_name).into();
    //     layout.Draw(None, &renderer, 0.0, 0.0).unwrap();

    //     let mut position = 0.0f32;
    //     let mut shaped_run = Vec::new();
    //     for (postscript_name, family_name, result) in renderer_inner.read().results.iter() {
    //         let font_info;
    //         let font_id;
    //         if let Some(id) = self.font_id_by_postscript_name.get(postscript_name) {
    //             font_id = *id;
    //         } else {
    //             font_id = self.select_font_by_family(family_name.clone()).unwrap();
    //         }
    //         font_info = &self.fonts[font_id.0];
    //         let mut glyph_runs = SmallVec::new();
    //         for glyph in result {
    //             glyph_runs.push(ShapedGlyph {
    //                 id: glyph.id,
    //                 position: point(px(position), px(0.0)),
    //                 index: *index_start,
    //                 is_emoji: font_info.is_emoji,
    //             });
    //             *index_start += 1;
    //             position += glyph.advance;
    //         }
    //         shaped_run.push(ShapedRun {
    //             font_id,
    //             glyphs: glyph_runs,
    //         });
    //     }

    //     let mut metrics = vec![DWRITE_LINE_METRICS::default(); 4];
    //     let mut line_count = 0u32;
    //     layout
    //         .GetLineMetrics(Some(&mut metrics), &mut line_count as _)
    //         .unwrap();
    //     *ascent = metrics[0].baseline;
    //     *descent = metrics[0].height - metrics[0].baseline;

    //     (position, shaped_run)
    // }

    // unsafe fn set_layout(&self, font_run: &FontRun, font_info: &FontInfo, offset: &mut usize, wstring_offset: &mut u32) {}

    fn layout_line(&mut self, text: &str, font_size: Pixels, font_runs: &[FontRun]) -> LineLayout {
        if font_runs.is_empty() {
            return LineLayout::default();
        }
        unsafe {
            let locale_wide = self
                .components
                .locale
                .encode_utf16()
                .chain(Some(0))
                .collect_vec();
            let locale_name = PCWSTR::from_raw(locale_wide.as_ptr());
            let text_wide = text.encode_utf16().collect_vec();

            let mut offset = 0usize;
            let mut wstring_offset = 0u32;
            let (text_format, text_layout) = {
                let first_run = &font_runs[0];
                let font_info = &self.fonts[first_run.font_id.0];
                let collection = {
                    let font_set = &self.font_sets[font_info.font_set_index];
                    self.components
                        .factory
                        .CreateFontCollectionFromFontSet(font_set)
                        .unwrap()
                };
                let font_family_name = font_info.font_family.clone();
                let font_weight = font_info.font_face.GetWeight();
                let font_style = font_info.font_face.GetStyle();
                let format = self
                    .components
                    .factory
                    .CreateTextFormat(
                        &HSTRING::from(&font_family_name),
                        &collection,
                        font_weight,
                        font_style,
                        DWRITE_FONT_STRETCH_NORMAL,
                        font_size.0,
                        locale_name,
                    )
                    .unwrap();
                let features = self.components.factory.CreateTypography().unwrap();
                for x in font_info.features.iter() {
                    features.AddFontFeature(*x).unwrap();
                }
                let layout = self
                    .components
                    .factory
                    .CreateTextLayout(&text_wide, &format, f32::INFINITY, f32::INFINITY)
                    .unwrap();
                let first_str = &text[offset..(offset + first_run.len)];
                offset += first_run.len;
                let first_wstring = first_str.encode_utf16().collect_vec();
                let local_length = first_wstring.len() as u32;
                let text_range = DWRITE_TEXT_RANGE {
                    startPosition: wstring_offset,
                    length: local_length,
                };
                layout.SetTypography(&features, text_range);
                wstring_offset += local_length;
                (format, layout)
            };

            let mut first_run = true;
            for run in font_runs {
                // if run.len == 0 {
                //     continue;
                // }
                if first_run {
                    //     let first_str = &text[offset..(offset + run.len)];
                    //     let first_wstring = first_str.encode_utf16().collect_vec();
                    //     wstring_offset += first_wstring.len() as u32;
                    first_run = false;
                    continue;
                }
                let font_info = &self.fonts[run.font_id.0];
                let local_str = &text[offset..(offset + run.len)];
                offset += run.len;
                let local_wide = local_str.encode_utf16().collect_vec();
                let local_length = local_wide.len() as u32;

                let collection = {
                    let font_set = &self.font_sets[font_info.font_set_index];
                    self.components
                        .factory
                        .CreateFontCollectionFromFontSet(font_set)
                        .unwrap()
                };
                let text_range = DWRITE_TEXT_RANGE {
                    startPosition: wstring_offset,
                    length: local_length,
                };
                wstring_offset += local_length;
                text_layout
                    .SetFontCollection(&collection, text_range)
                    .unwrap();
                text_layout
                    .SetFontFamilyName(&HSTRING::from(&font_info.font_family), text_range)
                    .unwrap();
                let features = self.components.factory.CreateTypography().unwrap();
                for x in font_info.features.iter() {
                    features.AddFontFeature(*x).unwrap();
                }
                text_layout.SetTypography(&features, text_range).unwrap();

                // let (position, result) = self.calculate_line_metrics(
                //     &mut index,
                //     &mut ascent,
                //     &mut descent,
                //     font_set_index,
                //     font_family_name.clone(),
                //     font_size.0,
                //     locale_name,
                //     &text_wide,
                //     font_weight,
                //     font_style,
                // );
                // glyph_position += position;
                // shaped_runs_vec.extend(result);
            }

            let renderer_inner = Arc::new(RwLock::new(TextRendererInner::new()));
            let renderer: IDWriteTextRenderer =
                TextRenderer::new(renderer_inner.clone(), locale_name).into();
            text_layout.Draw(None, &renderer, 0.0, 0.0).unwrap();
            let runs = {
                let mut vec = Vec::new();
                for result in renderer_inner.read().runs.iter() {
                    let font_id;
                    if let Some(id) = self.font_id_by_postscript_name.get(&result.postscript) {
                        font_id = *id;
                    } else {
                        font_id = self.select_font_by_family(result.family.clone()).unwrap();
                    }
                    let font_info = &self.fonts[font_id.0];
                    let mut glyphs = SmallVec::new();
                    for glyph in result.glyphs.iter() {
                        glyphs.push(ShapedGlyph {
                            id: glyph.id,
                            position: glyph.position,
                            index: glyph.index,
                            is_emoji: font_info.is_emoji,
                        });
                    }
                    vec.push(ShapedRun { font_id, glyphs });
                }
                vec
            };

            let mut metrics = vec![DWRITE_LINE_METRICS::default(); 4];
            let mut line_count = 0u32;
            text_layout
                .GetLineMetrics(Some(&mut metrics), &mut line_count as _)
                .unwrap();
            let width = renderer_inner.read().width;
            let ascent = px(metrics[0].baseline);
            let descent = px(metrics[0].height - metrics[0].baseline);

            LineLayout {
                font_size,
                width: px(width),
                ascent,
                descent,
                runs,
                len: text.len(),
            }
        }
    }

    fn font_metrics(&self, font_id: FontId) -> FontMetrics {
        unsafe {
            let font_info = &self.fonts[font_id.0];
            let mut metrics = std::mem::zeroed();
            font_info.font_face.GetMetrics2(&mut metrics);

            let res = FontMetrics {
                units_per_em: metrics.Base.designUnitsPerEm as _,
                ascent: metrics.Base.ascent as _,
                descent: -(metrics.Base.descent as f32),
                line_gap: metrics.Base.lineGap as _,
                underline_position: metrics.Base.underlinePosition as _,
                underline_thickness: metrics.Base.underlineThickness as _,
                cap_height: metrics.Base.capHeight as _,
                x_height: metrics.Base.xHeight as _,
                bounding_box: Bounds {
                    origin: Point {
                        x: metrics.glyphBoxLeft as _,
                        y: metrics.glyphBoxBottom as _,
                    },
                    size: Size {
                        width: (metrics.glyphBoxRight - metrics.glyphBoxLeft) as _,
                        height: (metrics.glyphBoxTop - metrics.glyphBoxBottom) as _,
                    },
                },
            };

            res
        }
    }

    unsafe fn get_glyphrun_analysis(
        &self,
        params: &RenderGlyphParams,
    ) -> windows::core::Result<IDWriteGlyphRunAnalysis> {
        let font = &self.fonts[params.font_id.0];
        let glyph_id = [params.glyph_id.0 as u16];
        let advance = [0.0f32];
        let offset = [DWRITE_GLYPH_OFFSET::default()];
        let glyph_run = DWRITE_GLYPH_RUN {
            fontFace: ManuallyDrop::new(Some(
                // TODO: remove this clone
                <IDWriteFontFace3 as Clone>::clone(&font.font_face).into(),
            )),
            fontEmSize: params.font_size.0,
            glyphCount: 1,
            glyphIndices: glyph_id.as_ptr(),
            glyphAdvances: advance.as_ptr(),
            glyphOffsets: offset.as_ptr(),
            isSideways: BOOL(0),
            bidiLevel: 0,
        };
        let transform = DWRITE_MATRIX {
            m11: params.scale_factor,
            m12: 0.0,
            m21: 0.0,
            m22: params.scale_factor,
            dx: 0.0,
            dy: 0.0,
        };
        self.components.factory.CreateGlyphRunAnalysis(
            &glyph_run as _,
            1.0,
            Some(&transform as _),
            // None,
            DWRITE_RENDERING_MODE_NATURAL,
            DWRITE_MEASURING_MODE_NATURAL,
            0.0,
            0.0,
        )
    }

    // unsafe fn get_glyphrun_analysis(
    //     &self,
    //     params: &RenderGlyphParams,
    // ) -> windows::core::Result<Vec<u8>> {
    //     let font = &self.fonts[params.font_id.0];
    //     let glyph_id = [params.glyph_id.0 as u16];
    //     let advance = [0.0f32];
    //     let offset = [DWRITE_GLYPH_OFFSET::default()];
    //     let glyph_run = DWRITE_GLYPH_RUN {
    //         fontFace: ManuallyDrop::new(Some(
    //             // TODO: remove this clone😀
    //             <IDWriteFontFace3 as Clone>::clone(&font.font_face).into(),
    //         )),
    //         fontEmSize: params.font_size.0,
    //         glyphCount: 1,
    //         glyphIndices: glyph_id.as_ptr(),
    //         glyphAdvances: advance.as_ptr(),
    //         glyphOffsets: offset.as_ptr(),
    //         isSideways: BOOL(0),
    //         bidiLevel: 0,
    //     };
    //     let transform = DWRITE_MATRIX {
    //         m11: params.scale_factor,
    //         m12: 0.0,
    //         m21: 0.0,
    //         m22: params.scale_factor,
    //         dx: 0.0,
    //         dy: 0.0,
    //     };
    //     if params.is_emoji {
    //         let enumerator = self.components.factory.TranslateColorGlyphRun(
    //             0.0,
    //             0.0,
    //             &glyph_run as _,
    //             None,
    //             DWRITE_MEASURING_MODE_NATURAL,
    //             Some(&transform as _),
    //             0,
    //         )?;
    //         enumerator.MoveNext().unwrap();
    //         let run = enumerator.GetCurrentRun()?;
    //         let emoji = &*run;
    //         let render_target_properties = D2D1_RENDER_TARGET_PROPERTIES {
    //             r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
    //             pixelFormat: D2D1_PIXEL_FORMAT {
    //                 format: DXGI_FORMAT_B8G8R8A8_UNORM,
    //                 alphaMode: D2D1_ALPHA_MODE_STRAIGHT,
    //             },
    //             dpiX: 96.0,
    //             dpiY: 96.0,
    //             usage: D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,
    //             minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
    //         };
    //         Ok(self
    //             .components
    //             .d2d1_factory
    //             .CreateDCRenderTarget(&render_target_properties)?)
    //     } else {
    //         self.components.factory.CreateGlyphRunAnalysis(
    //             &glyph_run as _,
    //             1.0,
    //             Some(&transform as _),
    //             // None,
    //             DWRITE_RENDERING_MODE_NATURAL,
    //             DWRITE_MEASURING_MODE_NATURAL,
    //             0.0,
    //             0.0,
    //         );
    //     }
    // }

    fn raster_bounds(&self, params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
        unsafe {
            let glyph_run_analysis = self.get_glyphrun_analysis(params)?;
            let bounds = glyph_run_analysis.GetAlphaTextureBounds(DWRITE_TEXTURE_CLEARTYPE_3x1)?;

            Ok(Bounds {
                origin: Point {
                    x: DevicePixels(bounds.left),
                    y: DevicePixels(bounds.top),
                },
                size: Size {
                    width: DevicePixels(bounds.right - bounds.left),
                    height: DevicePixels(bounds.bottom - bounds.top),
                },
            })
        }
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        let font_info = &self.fonts[font_id.0];
        let codepoints = [ch as u32];
        let mut glyph_indices = vec![0u16; 1];
        let ret = unsafe {
            font_info
                .font_face
                .GetGlyphIndices(codepoints.as_ptr(), 1, glyph_indices.as_mut_ptr())
                .log_err()
        }
        .map(|_| GlyphId(glyph_indices[0] as u32));
        ret
    }

    fn rasterize_glyph(
        &self,
        params: &RenderGlyphParams,
        glyph_bounds: Bounds<DevicePixels>,
    ) -> Result<(Size<DevicePixels>, Vec<u8>)> {
        if glyph_bounds.size.width.0 == 0 || glyph_bounds.size.height.0 == 0 {
            return Err(anyhow!("glyph bounds are empty"));
        }
        let font_info = &self.fonts[params.font_id.0];
        println!(
            "rastering: {}, is emoji {}",
            font_info.font_family, params.is_emoji
        );
        let glyph_id = [params.glyph_id.0 as u16];
        let advance = [0.0f32];
        let offset = [DWRITE_GLYPH_OFFSET::default()];
        let glyph_run = DWRITE_GLYPH_RUN {
            fontFace: ManuallyDrop::new(Some(
                // TODO: remove this clone😀
                <IDWriteFontFace3 as Clone>::clone(&font_info.font_face).into(),
            )),
            fontEmSize: params.font_size.0,
            glyphCount: 1,
            glyphIndices: glyph_id.as_ptr(),
            glyphAdvances: advance.as_ptr(),
            glyphOffsets: offset.as_ptr(),
            isSideways: BOOL(0),
            bidiLevel: 0,
        };
        let transform = DWRITE_MATRIX {
            m11: params.scale_factor,
            m12: 0.0,
            m21: 0.0,
            m22: params.scale_factor,
            dx: 0.0,
            dy: 0.0,
        };
        let subpixel_shift = params
            .subpixel_variant
            .map(|v| v as f32 / SUBPIXEL_VARIANTS as f32);
        println!("Subpixel shift: {:#?}", subpixel_shift);
        unsafe {
            if params.is_emoji {
                // TODO:
                // let mut bitmap_size = glyph_bounds.size;
                // if params.subpixel_variant.x > 0 {
                //     bitmap_size.width += DevicePixels(1);
                // }
                // if params.subpixel_variant.y > 0 {
                //     bitmap_size.height += DevicePixels(1);
                // }
                // let bitmap_size = bitmap_size;
                let bitmap_size = glyph_bounds.size;
                let total_bytes = bitmap_size.height.0 as usize * bitmap_size.width.0 as usize * 4;
                let texture_bounds = RECT {
                    left: glyph_bounds.left().0,
                    top: glyph_bounds.top().0,
                    right: glyph_bounds.right().0,
                    bottom: glyph_bounds.bottom().0,
                };

                let mut bitmap = vec![0u8; total_bytes];
                let enumerator = self
                    .components
                    .factory
                    .TranslateColorGlyphRun2(
                        D2D_POINT_2F { x: 0.0, y: 0.0 },
                        &glyph_run as _,
                        None,
                        DWRITE_GLYPH_IMAGE_FORMATS_COLR,
                        DWRITE_MEASURING_MODE_NATURAL,
                        Some(&transform as _),
                        0,
                    )
                    .unwrap();

                let bitmap_hdc = self.components.render_target.GetMemoryDC();
                SetBoundsRect(
                    bitmap_hdc,
                    Some(&texture_bounds),
                    SET_BOUNDS_RECT_FLAGS(DCB_ENABLE.0 | DCB_RESET.0),
                );
                // clear the bitmap
                {
                    let size = self.components.render_target.GetSize().unwrap();
                    println!("Bitmap size: {:#?}", size);
                    SetDCBrushColor(bitmap_hdc, COLORREF(0xFFFFFF));
                    SelectObject(bitmap_hdc, GetStockObject(NULL_PEN));
                    SelectObject(bitmap_hdc, GetStockObject(DC_BRUSH));
                    Rectangle(bitmap_hdc, 0, 0, size.cx + 1, size.cy + 1);
                }

                let current_transform = DWRITE_MATRIX {
                    m11: 1.0,
                    m12: 0.0,
                    m21: 0.0,
                    m22: 1.0,
                    dx: -(glyph_bounds.origin.x.0 / 2) as f32,
                    dy: ((glyph_bounds.origin.y.0 + glyph_bounds.size.height.0) / 2) as f32,
                };
                println!("Trans: {:#?}", current_transform);
                self.components
                    .render_target
                    .SetCurrentTransform(Some(&current_transform));

                let render_params = self.components.factory.CreateRenderingParams()?;
                // bitmap_render_target
                //     .DrawGlyphRunWithColorSupport(
                //         0.0,
                //         0.0,
                //         DWRITE_MEASURING_MODE_NATURAL,
                //         &glyph_run,
                //         &render_params,
                //         COLORREF(0x77777777),
                //         0,
                //         None,
                //     )
                //     .inspect_err(|e| {
                //         println!("Error: {}, msg: {}", e, std::io::Error::last_os_error())
                //     });

                while enumerator.MoveNext().is_ok() {
                    let Ok(run) = enumerator.GetCurrentRun2() else {
                        break;
                    };
                    let emoji = &*run;
                    self.components
                        .render_target
                        .DrawGlyphRun(
                            0.0,
                            (glyph_bounds.size.height.0 / 2) as f32,
                            DWRITE_MEASURING_MODE_NATURAL,
                            &emoji.Base.glyphRun,
                            &render_params,
                            translate_color(&emoji.Base.runColor),
                            None,
                        )
                        .unwrap();
                }

                let mut raw_bytes = vec![0u8; total_bytes];
                let bitmap_data = self.components.render_target.GetBitmapData().unwrap();
                let raw_u32 = std::slice::from_raw_parts(bitmap_data.pixels, total_bytes / 4);
                for (bytes, color) in raw_bytes.chunks_exact_mut(4).zip(raw_u32.iter()) {
                    bytes[3] = 0xFF;
                    if *color == 0 {
                        continue;
                    }
                    bytes[0] = (color >> 16 & 0xFF) as u8;
                    bytes[1] = (color >> 8 & 0xFF) as u8;
                    bytes[2] = (color & 0xFF) as u8;
                }
                Ok((bitmap_size, raw_bytes))
            } else {
                let bitmap_size = glyph_bounds.size;

                let glyph_run_analysis = self.get_glyphrun_analysis(params)?;
                let total_bytes = bitmap_size.height.0 as usize * bitmap_size.width.0 as usize * 3;
                let texture_bounds = RECT {
                    left: glyph_bounds.left().0,
                    top: glyph_bounds.top().0,
                    right: glyph_bounds.right().0,
                    bottom: glyph_bounds.bottom().0,
                };
                let mut result = vec![0u8; total_bytes];
                glyph_run_analysis.CreateAlphaTexture(
                    DWRITE_TEXTURE_CLEARTYPE_3x1,
                    &texture_bounds as _,
                    &mut result,
                )?;
                let mut bitmap_rawdata =
                    vec![0u8; bitmap_size.height.0 as usize * bitmap_size.width.0 as usize];
                for (chunk, num) in result.chunks_exact(3).zip(bitmap_rawdata.iter_mut()) {
                    let sum: u32 = chunk.iter().map(|&x| x as u32).sum();
                    *num = (sum / 3) as u8;
                }
                Ok((bitmap_size, bitmap_rawdata))

                // let bitmap_size = glyph_bounds.size;
                // let total_bytes = bitmap_size.height.0 as usize * bitmap_size.width.0 as usize * 4;
                // let total_bytes = bitmap_size.height.0 as usize * bitmap_size.width.0 as usize;
                // let texture_bounds = RECT {
                //     left: glyph_bounds.left().0,
                //     top: glyph_bounds.top().0,
                //     right: glyph_bounds.left().0 + bitmap_size.width.0,
                //     bottom: glyph_bounds.top().0 + bitmap_size.height.0,
                // };

                // let gdi = self.components.factory.GetGdiInterop()?;
                // let bitmap_render_target = gdi.CreateBitmapRenderTarget(
                //     None,
                //     bitmap_size.width.0 as _,
                //     bitmap_size.height.0 as _,
                // )?;
                // bitmap_render_target.SetCurrentTransform(Some(&transform));
                // let bitmap_render_target: IDWriteBitmapRenderTarget3 =
                //     std::mem::transmute(bitmap_render_target);
                // bitmap_render_target.DrawGlyphRun(0.0, 0.0, measuringmode, glyphrun, renderingparams, textcolor, blackboxrect).unwrap();
                // let raw_bytes =
                //     Vec::from_raw_parts(bitmap_rawdata.pixels as *mut u8, total_bytes, total_bytes);
                // let raw_bytes = Vec::from_raw_parts(bitmap_rawdata.bits, total_bytes, total_bytes);

                // let mut res = vec![0u8; total_bytes / 4];
                // for (chunk, num) in raw_bytes.chunks_exact(4).zip(res.iter_mut()) {
                //     let sum: u32 = chunk.iter().map(|&x| x as u32).sum();
                //     *num = (sum / 3) as u8;
                // }
                // Ok((bitmap_size, res))

                // bitmap.Unmap().unwrap();
                // Ok((bitmap_size, raw_bytes))
            }
        }
    }

    fn get_typographic_bounds(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Bounds<f32>> {
        unsafe {
            let font = &self.fonts[font_id.0].font_face;
            let glyph_indices = [glyph_id.0 as u16];
            let mut metrics = [DWRITE_GLYPH_METRICS::default()];
            font.GetDesignGlyphMetrics(glyph_indices.as_ptr(), 1, metrics.as_mut_ptr(), false)?;

            let metrics = &metrics[0];
            let advance_width = metrics.advanceWidth as i32;
            let advance_height = metrics.advanceHeight as i32;
            let left_side_bearing = metrics.leftSideBearing as i32;
            let right_side_bearing = metrics.rightSideBearing as i32;
            let top_side_bearing = metrics.topSideBearing as i32;
            let bottom_side_bearing = metrics.bottomSideBearing as i32;
            let vertical_origin_y = metrics.verticalOriginY as i32;

            let y_offset = vertical_origin_y + bottom_side_bearing - advance_height;
            let width = advance_width - (left_side_bearing + right_side_bearing);
            let height = advance_height - (top_side_bearing + bottom_side_bearing);

            Ok(Bounds {
                origin: Point {
                    x: left_side_bearing as f32,
                    y: y_offset as f32,
                },
                size: Size {
                    width: width as f32,
                    height: height as f32,
                },
            })
        }
    }

    fn get_advance(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Size<f32>> {
        unsafe {
            let font = &self.fonts[font_id.0].font_face;
            let glyph_indices = [glyph_id.0 as u16];
            let mut metrics = [DWRITE_GLYPH_METRICS::default()];
            font.GetDesignGlyphMetrics(glyph_indices.as_ptr(), 1, metrics.as_mut_ptr(), false)?;

            let metrics = &metrics[0];

            Ok(Size {
                width: metrics.advanceWidth as f32,
                height: 0.0,
            })
        }
    }

    fn all_font_names(&self) -> Vec<String> {
        unsafe {
            let mut result = Vec::new();
            let mut system_collection = std::mem::zeroed();
            self.components
                .factory
                .GetSystemFontCollection(&mut system_collection, false)
                .unwrap();
            if system_collection.is_none() {
                return result;
            }
            let system_collection = system_collection.unwrap();
            let locale_name_wide = self
                .components
                .locale
                .encode_utf16()
                .chain(Some(0))
                .collect_vec();
            let locale_name = PCWSTR::from_raw(locale_name_wide.as_ptr());
            let family_count = system_collection.GetFontFamilyCount();
            for index in 0..family_count {
                let font_family = system_collection.GetFontFamily(index).unwrap();
                let font_count = font_family.GetFontCount();
                for font_index in 0..font_count {
                    let font = font_family.GetFont(font_index).unwrap();
                    let mut font_name_localized_string: Option<IDWriteLocalizedStrings> = {
                        let mut string: Option<IDWriteLocalizedStrings> = std::mem::zeroed();
                        let mut exists = BOOL(0);
                        font.GetInformationalStrings(
                            DWRITE_INFORMATIONAL_STRING_FULL_NAME,
                            &mut string as _,
                            &mut exists as _,
                        )
                        .unwrap();
                        if exists.as_bool() {
                            string
                        } else {
                            continue;
                        }
                    };
                    let Some(localized_font_name) = font_name_localized_string else {
                        continue;
                    };
                    let Some(font_name) = get_name(localized_font_name, locale_name) else {
                        continue;
                    };
                    result.push(font_name);
                }
            }

            result
        }
    }

    fn all_font_families(&self) -> Vec<String> {
        unsafe {
            let mut result = Vec::new();
            let mut system_collection = std::mem::zeroed();
            self.components
                .factory
                .GetSystemFontCollection(&mut system_collection, false)
                .unwrap();
            if system_collection.is_none() {
                return result;
            }
            let system_collection = system_collection.unwrap();
            let locale_name_wide = self
                .components
                .locale
                .encode_utf16()
                .chain(Some(0))
                .collect_vec();
            let locale_name = PCWSTR::from_raw(locale_name_wide.as_ptr());
            let family_count = system_collection.GetFontFamilyCount();
            for index in 0..family_count {
                let Some(font_family) = system_collection.GetFontFamily(index).log_err() else {
                    continue;
                };
                let Some(localized_family_name) = font_family.GetFamilyNames().log_err() else {
                    continue;
                };
                let Some(family_name) = get_name(localized_family_name, locale_name) else {
                    continue;
                };
                result.push(family_name);
            }

            result
        }
    }
}

impl Drop for DirectWriteState {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .components
                .factory
                .UnregisterFontFileLoader(&self.components.in_memory_loader);
        }
    }
}

// #[implement(IDWriteTextAnalysisSource, IDWriteTextAnalysisSink)]
struct Analysis {
    source: IDWriteTextAnalysisSource,
    sink: IDWriteTextAnalysisSink,
    sink_inner: Arc<RwLock<AnalysisSinkInner>>,
    length: u32,
}

#[implement(IDWriteTextAnalysisSource)]
struct AnalysisSource {
    locale: PCWSTR,
    text: Vec<u16>,
    text_length: u32,
}

#[implement(IDWriteTextAnalysisSink)]
struct AnalysisSink {
    inner: Arc<RwLock<AnalysisSinkInner>>,
}

struct AnalysisSinkInner {
    results: Vec<AnalysisResult>,
}

#[derive(Clone, Debug)]
struct AnalysisResult {
    text_position: u32,
    test_length: u32,
    script_analysis: DWRITE_SCRIPT_ANALYSIS,
}

impl AnalysisSource {
    pub fn new(locale: PCWSTR, text: Vec<u16>, text_length: u32) -> Self {
        AnalysisSource {
            locale,
            text,
            text_length,
        }
    }
}

impl AnalysisSink {
    pub fn new(inner: Arc<RwLock<AnalysisSinkInner>>) -> Self {
        AnalysisSink { inner }
    }
}

impl AnalysisSinkInner {
    pub fn new() -> Self {
        AnalysisSinkInner {
            results: Vec::new(),
        }
    }

    pub fn get_result(&self) -> Vec<AnalysisResult> {
        self.results.clone()
    }
}

impl Analysis {
    pub fn new(locale: PCWSTR, text: Vec<u16>, text_length: u32) -> Self {
        let source_struct = AnalysisSource::new(locale, text, text_length);
        let sink_inner = Arc::new(RwLock::new(AnalysisSinkInner::new()));
        let sink_struct = AnalysisSink::new(sink_inner.clone());
        let source: IDWriteTextAnalysisSource = source_struct.into();
        let sink: IDWriteTextAnalysisSink = sink_struct.into();

        Analysis {
            source,
            sink,
            sink_inner,
            length: text_length,
        }
    }

    // https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextanalyzer-getglyphs
    pub unsafe fn generate_result(&self, analyzer: &IDWriteTextAnalyzer) -> Vec<AnalysisResult> {
        analyzer
            .AnalyzeScript(&self.source, 0, self.length, &self.sink)
            .unwrap();
        self.sink_inner.read().get_result()
    }
}

// https://github.com/microsoft/Windows-classic-samples/blob/main/Samples/Win7Samples/multimedia/DirectWrite/CustomLayout/TextAnalysis.cpp
impl IDWriteTextAnalysisSource_Impl for AnalysisSource {
    fn GetTextAtPosition(
        &self,
        textposition: u32,
        textstring: *mut *mut u16,
        textlength: *mut u32,
    ) -> windows::core::Result<()> {
        if textposition >= self.text_length {
            unsafe {
                *textstring = std::ptr::null_mut() as _;
                *textlength = 0;
            }
        } else {
            unsafe {
                // *textstring = self.text.as_wide()[textposition as usize..].as_ptr() as *mut u16;
                *textstring = self.text.as_ptr().add(textposition as usize) as _;
                *textlength = self.text_length - textposition;
            }
        }
        Ok(())
    }

    fn GetTextBeforePosition(
        &self,
        textposition: u32,
        textstring: *mut *mut u16,
        textlength: *mut u32,
    ) -> windows::core::Result<()> {
        if textposition == 0 || textposition >= self.text_length {
            unsafe {
                *textstring = 0 as _;
                *textlength = 0;
            }
        } else {
            unsafe {
                *textstring = self.text.as_ptr() as *mut u16;
                *textlength = textposition - 0;
            }
        }
        Ok(())
    }

    fn GetParagraphReadingDirection(&self) -> DWRITE_READING_DIRECTION {
        DWRITE_READING_DIRECTION_LEFT_TO_RIGHT
    }

    fn GetLocaleName(
        &self,
        textposition: u32,
        textlength: *mut u32,
        localename: *mut *mut u16,
    ) -> windows::core::Result<()> {
        unsafe {
            *localename = self.locale.as_ptr() as *mut u16;
            *textlength = self.text_length - textposition;
        }
        Ok(())
    }

    fn GetNumberSubstitution(
        &self,
        _textposition: u32,
        _textlength: *mut u32,
        _numbersubstitution: *mut Option<IDWriteNumberSubstitution>,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            HRESULT(-1),
            "GetNumberSubstitution unimplemented",
        ))
    }
}

impl IDWriteTextAnalysisSink_Impl for AnalysisSink {
    fn SetScriptAnalysis(
        &self,
        textposition: u32,
        textlength: u32,
        scriptanalysis: *const DWRITE_SCRIPT_ANALYSIS,
    ) -> windows::core::Result<()> {
        let mut inner = self.inner.write();
        unsafe {
            inner.results.push(AnalysisResult {
                text_position: textposition,
                test_length: textlength,
                script_analysis: *scriptanalysis,
            });
        }
        Ok(())
    }

    fn SetLineBreakpoints(
        &self,
        _textposition: u32,
        _textlength: u32,
        _linebreakpoints: *const DWRITE_LINE_BREAKPOINT,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            HRESULT(-1),
            "SetLineBreakpoints unimplemented",
        ))
    }

    fn SetBidiLevel(
        &self,
        _textposition: u32,
        _textlength: u32,
        _explicitlevel: u8,
        _resolvedlevel: u8,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            HRESULT(-1),
            "SetBidiLevel unimplemented",
        ))
    }

    fn SetNumberSubstitution(
        &self,
        _textposition: u32,
        _textlength: u32,
        _numbersubstitution: Option<&IDWriteNumberSubstitution>,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            HRESULT(-1),
            "SetNumberSubstitution unimplemented",
        ))
    }
}

#[implement(IDWriteTextRenderer)]
struct TextRenderer {
    inner: Arc<RwLock<TextRendererInner>>,
    locale: PCWSTR,
}

impl TextRenderer {
    pub fn new(inner: Arc<RwLock<TextRendererInner>>, locale: PCWSTR) -> Self {
        TextRenderer { inner, locale }
    }
}

struct RendererShapedGlyph {
    id: GlyphId,
    position: Point<Pixels>,
    index: usize,
}

struct RendererShapedRun {
    postscript: String,
    family: String,
    glyphs: SmallVec<[RendererShapedGlyph; 8]>,
}

struct TextRendererInner {
    index: usize,
    width: f32,
    runs: Vec<RendererShapedRun>,
}

impl TextRendererInner {
    pub fn new() -> Self {
        TextRendererInner {
            index: 0,
            width: 0.0,
            runs: Vec::new(),
        }
    }
}

struct GlyphRunResult {
    id: GlyphId,
    advance: f32,
    index: usize,
}

impl IDWritePixelSnapping_Impl for TextRenderer {
    fn IsPixelSnappingDisabled(
        &self,
        _clientdrawingcontext: *const ::core::ffi::c_void,
    ) -> windows::core::Result<BOOL> {
        Ok(BOOL(1))
    }

    fn GetCurrentTransform(
        &self,
        _clientdrawingcontext: *const ::core::ffi::c_void,
        transform: *mut DWRITE_MATRIX,
    ) -> windows::core::Result<()> {
        unsafe {
            *transform = DWRITE_MATRIX {
                m11: 1.0,
                m12: 0.0,
                m21: 0.0,
                m22: 1.0,
                dx: 0.0,
                dy: 0.0,
            };
        }
        Ok(())
    }

    fn GetPixelsPerDip(
        &self,
        _clientdrawingcontext: *const ::core::ffi::c_void,
    ) -> windows::core::Result<f32> {
        Ok(1.0)
    }
}

impl IDWriteTextRenderer_Impl for TextRenderer {
    fn DrawGlyphRun(
        &self,
        _clientdrawingcontext: *const ::core::ffi::c_void,
        _baselineoriginx: f32,
        _baselineoriginy: f32,
        _measuringmode: DWRITE_MEASURING_MODE,
        glyphrun: *const DWRITE_GLYPH_RUN,
        glyphrundescription: *const DWRITE_GLYPH_RUN_DESCRIPTION,
        _clientdrawingeffect: Option<&windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        unsafe {
            let glyphrun = &*glyphrun;
            let desc = &*glyphrundescription;

            if glyphrun.fontFace.is_none() {
                return Ok(());
            }
            let font = glyphrun.fontFace.as_ref().unwrap();
            let Some((postscript_name, family_name)) =
                get_postscript_and_family_name(font, self.locale)
            else {
                log::error!("none postscript name found");
                return Ok(());
            };

            let mut global_index = self.inner.read().index;
            let mut position = self.inner.read().width;
            let mut glyphs = SmallVec::new();
            for index in 0..glyphrun.glyphCount {
                let id = GlyphId(*glyphrun.glyphIndices.add(index as _) as u32);
                glyphs.push(RendererShapedGlyph {
                    id,
                    position: point(px(position), px(0.0)),
                    index: global_index,
                });
                position += *glyphrun.glyphAdvances.add(index as _);
                global_index += 1;
            }
            self.inner.write().index = global_index;
            self.inner.write().width = position;
            self.inner.write().runs.push(RendererShapedRun {
                postscript: postscript_name,
                family: family_name,
                glyphs,
            });
        }
        Ok(())
    }

    fn DrawUnderline(
        &self,
        _clientdrawingcontext: *const ::core::ffi::c_void,
        _baselineoriginx: f32,
        _baselineoriginy: f32,
        _underline: *const DWRITE_UNDERLINE,
        _clientdrawingeffect: Option<&windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            HRESULT(-1),
            "DrawUnderline unimplemented",
        ))
    }

    fn DrawStrikethrough(
        &self,
        _clientdrawingcontext: *const ::core::ffi::c_void,
        _baselineoriginx: f32,
        _baselineoriginy: f32,
        _strikethrough: *const DWRITE_STRIKETHROUGH,
        _clientdrawingeffect: Option<&windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            HRESULT(-1),
            "DrawStrikethrough unimplemented",
        ))
    }

    fn DrawInlineObject(
        &self,
        _clientdrawingcontext: *const ::core::ffi::c_void,
        _originx: f32,
        _originy: f32,
        _inlineobject: Option<&IDWriteInlineObject>,
        _issideways: BOOL,
        _isrighttoleft: BOOL,
        _clientdrawingeffect: Option<&windows::core::IUnknown>,
    ) -> windows::core::Result<()> {
        Err(windows::core::Error::new(
            HRESULT(-1),
            "DrawInlineObject unimplemented",
        ))
    }
}

unsafe fn get_postscript_and_family_name(
    font_face: &IDWriteFontFace,
    locale: PCWSTR,
) -> Option<(String, String)> {
    let font_face_pointer = font_face as *const IDWriteFontFace;
    let font_face_3_pointer: *const IDWriteFontFace3 = std::mem::transmute(font_face_pointer);
    let font_face_3 = &*font_face_3_pointer;
    let Some(postscript_name) = get_postscript_name(font_face_3) else {
        return None;
    };
    let Some(localized_family_name) = font_face_3.GetFamilyNames().log_err() else {
        return None;
    };
    Some((
        postscript_name,
        get_name(localized_family_name, locale).unwrap(),
    ))
}

unsafe fn get_postscript_name(font_face: &IDWriteFontFace3) -> Option<String> {
    let mut info = std::mem::zeroed();
    let mut exists = BOOL(0);
    font_face
        .GetInformationalStrings(
            DWRITE_INFORMATIONAL_STRING_POSTSCRIPT_NAME,
            &mut info,
            &mut exists,
        )
        .unwrap();
    if !exists.as_bool() || info.is_none() {
        return None;
    }

    get_name(info.unwrap(), DEFAULT_LOCALE_NAME)
}

// https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ne-dwrite-dwrite_font_feature_tag
fn direct_write_features(features: &FontFeatures) -> Vec<DWRITE_FONT_FEATURE> {
    let mut feature_list = Vec::new();
    let tag_values = features.tag_value_list();
    if tag_values.is_empty() {
        return feature_list;
    }
    // All of these features are enabled by default by DirectWrite.
    // If you want to (and can) peek into the source of DirectWrite
    add_feature(&mut feature_list, "liga", true);
    add_feature(&mut feature_list, "clig", true);
    add_feature(&mut feature_list, "calt", true);

    for (tag, enable) in tag_values {
        if tag == "liga".to_string() && !enable {
            feature_list[0].parameter = 0;
            continue;
        }
        if tag == "clig".to_string() && !enable {
            feature_list[1].parameter = 0;
            continue;
        }
        if tag == "calt".to_string() && !enable {
            feature_list[2].parameter = 0;
            continue;
        }
        add_feature(&mut feature_list, &tag, enable);
    }

    feature_list
}

fn add_feature(feature_list: &mut Vec<DWRITE_FONT_FEATURE>, feature_name: &str, enable: bool) {
    let tag = make_direct_write_tag(feature_name);
    let font_feature = if enable {
        DWRITE_FONT_FEATURE {
            nameTag: tag,
            parameter: 1,
        }
    } else {
        DWRITE_FONT_FEATURE {
            nameTag: tag,
            parameter: 0,
        }
    };
    feature_list.push(font_feature);
}

#[inline]
fn make_open_type_tag(tag_name: &str) -> u32 {
    assert_eq!(tag_name.chars().count(), 4);
    let bytes = tag_name.bytes().collect_vec();
    ((bytes[3] as u32) << 24)
        | ((bytes[2] as u32) << 16)
        | ((bytes[1] as u32) << 8)
        | (bytes[0] as u32)
}

#[inline]
fn make_direct_write_tag(tag_name: &str) -> DWRITE_FONT_FEATURE_TAG {
    DWRITE_FONT_FEATURE_TAG(make_open_type_tag(tag_name))
}

unsafe fn get_name(string: IDWriteLocalizedStrings, locale: PCWSTR) -> Option<String> {
    let mut locale_name_index = 0u32;
    let mut exists = BOOL(0);
    string
        .FindLocaleName(locale, &mut locale_name_index, &mut exists as _)
        .unwrap();
    if !exists.as_bool() {
        string
            .FindLocaleName(
                DEFAULT_LOCALE_NAME,
                &mut locale_name_index as _,
                &mut exists as _,
            )
            .unwrap();
    }
    if !exists.as_bool() {
        return None;
    }

    let name_length = string.GetStringLength(locale_name_index).unwrap() as usize;
    let mut name_vec = vec![0u16; name_length + 1];
    string.GetString(locale_name_index, &mut name_vec).unwrap();

    Some(String::from_utf16_lossy(&name_vec[..name_length]))
}

fn translate_color(color: &DWRITE_COLOR_F) -> COLORREF {
    let r_int = (color.r * 255.0) as u32;
    let g_int = (color.g * 255.0) as u32;
    let b_int = (color.b * 255.0) as u32;
    let a_int = (color.a * 255.0) as u32;

    let color_ref = (b_int << 16) | (g_int << 8) | r_int;
    if color_ref != 0 {
        println!("RGB {:?} => color {}", color, color_ref);
    }
    COLORREF((a_int << 24) | (b_int << 16) | (g_int << 8) | r_int)
}

const DEFAULT_LOCALE_NAME: PCWSTR = windows::core::w!("en-US");
