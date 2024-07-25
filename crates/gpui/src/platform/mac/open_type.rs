#![allow(unused, non_upper_case_globals)]

use crate::FontFeatures;
use cocoa::appkit::CGFloat;
use core_foundation::{
    array::{
        kCFTypeArrayCallBacks, CFArray, CFArrayAppendValue, CFArrayCreateMutable, CFArrayRef,
        CFMutableArrayRef,
    },
    base::{kCFAllocatorDefault, CFRelease, TCFType, TCFTypeRef},
    dictionary::{
        kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks, CFDictionaryCreate,
    },
    number::CFNumber,
    string::{CFString, CFStringRef},
};
use core_graphics::{display::CFDictionary, geometry::CGAffineTransform};
use core_text::{
    font::{CTFont, CTFontRef},
    font_descriptor::{
        kCTFontFeatureSettingsAttribute, CTFontDescriptor, CTFontDescriptorCopyAttributes,
        CTFontDescriptorCreateCopyWithFeature, CTFontDescriptorCreateWithAttributes,
        CTFontDescriptorRef,
    },
};
use font_kit::font::Font;
use std::ptr;

pub fn apply_features(font: &mut Font, features: &FontFeatures) {
    unsafe {
        let native_font = font.native_font();
        let mut feature_array =
            CFArrayCreateMutable(kCFAllocatorDefault, 0, &kCFTypeArrayCallBacks);
        for (tag, value) in features.tag_value_list() {
            let keys = [kCTFontOpenTypeFeatureTag, kCTFontOpenTypeFeatureValue];
            let values = [
                CFString::new(&tag).as_CFTypeRef(),
                CFNumber::from(*value as i32).as_CFTypeRef(),
            ];
            let dict = CFDictionaryCreate(
                kCFAllocatorDefault,
                &keys as *const _ as _,
                &values as *const _ as _,
                2,
                &kCFTypeDictionaryKeyCallBacks,
                &kCFTypeDictionaryValueCallBacks,
            );
            values.into_iter().for_each(|value| CFRelease(value));
            CFArrayAppendValue(feature_array, dict as _);
            CFRelease(dict as _);
        }
        let attrs = CFDictionaryCreate(
            kCFAllocatorDefault,
            &kCTFontFeatureSettingsAttribute as *const _ as _,
            &feature_array as *const _ as _,
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );
        CFRelease(feature_array as *const _ as _);
        let new_descriptor = CTFontDescriptorCreateWithAttributes(attrs);
        CFRelease(attrs as _);
        let new_descriptor = CTFontDescriptor::wrap_under_create_rule(new_descriptor);
        let new_font = CTFontCreateCopyWithAttributes(
            font.native_font().as_concrete_TypeRef(),
            0.0,
            ptr::null(),
            new_descriptor.as_concrete_TypeRef(),
        );
        let new_font = CTFont::wrap_under_create_rule(new_font);
        *font = Font::from_native_font(&new_font);
    }
}

pub fn retrieve_font_features(font: &Font) -> Vec<String> {
    unsafe {
        let mut result = Vec::new();
        let native_font = font.native_font();
        let array = CTFontCopyFeatures(native_font.as_concrete_TypeRef());
        let features: CFArray<CFDictionary<CFString>> = CFArray::wrap_under_get_rule(array);
        for feature in features.iter() {
            if let Some(feature_tag) = feature.find(kCTFontOpenTypeFeatureTag) {
                let tag = CFString::wrap_under_get_rule(*feature_tag as _).to_string();
                result.push(tag);
            } else if let Some(selector_dict) = feature.find(kCTFontFeatureTypeSelectorsKey) {
                let selector_dict: CFArray<CFDictionary<CFString>> =
                    CFArray::wrap_under_get_rule(*selector_dict as _);
                for selector in selector_dict.iter() {
                    let Some(feature_tag) = selector.find(kCTFontOpenTypeFeatureTag) else {
                        continue;
                    };
                    let tag = CFString::wrap_under_get_rule(*feature_tag as _).to_string();
                    result.push(tag);
                }
            }
        }
        result
    }
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
    static kCTFontOpenTypeFeatureTag: CFStringRef;
    static kCTFontOpenTypeFeatureValue: CFStringRef;
    static kCTFontFeatureTypeSelectorsKey: CFStringRef;

    fn CTFontCreateCopyWithAttributes(
        font: CTFontRef,
        size: CGFloat,
        matrix: *const CGAffineTransform,
        attributes: CTFontDescriptorRef,
    ) -> CTFontRef;

    fn CTFontCopyFeatures(font: CTFontRef) -> CFArrayRef;
}
