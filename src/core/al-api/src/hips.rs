use al_core::WebGlContext;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use super::blend::BlendCfg;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct CompositeHiPS {
    hipses: Vec<SimpleHiPS>,
}

use std::iter::IntoIterator;
impl IntoIterator for CompositeHiPS {
    type Item = SimpleHiPS;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.hipses.into_iter()
    }
}

#[derive(Deserialize, Debug)]
pub struct SimpleHiPS {
    /// The HiPS metadata
    pub properties: HiPSProperties,

    /// Color config
    #[serde(default)]
    pub color: HiPSColor,

    // Blending config
    #[serde(default)]
    pub blend_cfg: BlendCfg,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_opacity() -> f32 {
    1.0
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HiPSProperties {
    pub url: String,

    pub max_order: u8,
    pub frame: Frame,
    pub tile_size: i32,
    pub min_cutout: Option<f32>,
    pub max_cutout: Option<f32>,
    pub format: HiPSFormat,
}

#[derive(Deserialize, Debug)]
pub struct Frame {
    pub label: String,
    pub system: String,
}

#[derive(Deserialize, Debug)]
pub enum HiPSFormat {
    FITSImage { bitpix: i32 },
    Image { format: String },
}

#[derive(Deserialize, Debug, Clone)]
pub enum HiPSColor {
    Grayscale2Colormap {
        colormap: String,
        transfer: String,
        reversed: bool,
    },
    Grayscale2Color {
        color: [f32; 3],
        transfer: String,
        k: f32,
    },
    Color,
}

impl Default for HiPSColor {
    fn default() -> Self {
        HiPSColor::Grayscale2Color {
            color: [1.0, 0.0, 0.0],
            transfer: String::from("asinh"),
            k: 1.0
        }
    }
}