use al_core::image::format::RGBA32F;
use al_core::{image::format::ImageFormat, image::raw::ImageBuffer};
#[derive(Debug)]
pub struct EmptyTileImage {
    inner: ImageType,
}

use al_core::{pixel::Pixel, image::ImageType};
impl EmptyTileImage {
    fn new(size: i32, format: ImageFormatType) -> EmptyTileImage {
        debug_assert!(math::utils::is_power_of_two(size as usize));
        let inner = match format {
            ImageFormatType::RGBA8U => {
                let image = ImageBuffer::<RGBA8U>::allocate(&<<RGBA8U as ImageFormat>::P as Pixel>::BLACK, size, size);
                ImageType::RawRgba8u { image }
            },
            ImageFormatType::RGB8U => {
                let image = ImageBuffer::<RGB8U>::allocate(&<<RGB8U as ImageFormat>::P as Pixel>::BLACK, size, size);
                ImageType::RawRgb8u { image }
            },
            ImageFormatType::R32F => {
                let image = ImageBuffer::<R32F>::allocate(&<<R32F as ImageFormat>::P as Pixel>::BLACK, size, size);
                ImageType::RawR32f { image }
            },
            #[cfg(feature = "webgl2")]
            ImageFormatType::R8UI => {
                let image = ImageBuffer::<R8UI>::allocate(&<<R8UI as ImageFormat>::P as Pixel>::BLACK, size, size);
                ImageType::RawR8ui { image }
            },
            #[cfg(feature = "webgl2")]
            ImageFormatType::R16I => {
                let image = ImageBuffer::<R16I>::allocate(&<<R16I as ImageFormat>::P as Pixel>::BLACK, size, size);
                ImageType::RawR16i { image }
            },
            #[cfg(feature = "webgl2")]
            ImageFormatType::R32I => {
                let image = ImageBuffer::<R32I>::allocate(&<<R32I as ImageFormat>::P as Pixel>::BLACK, size, size);
                ImageType::RawR32i { image }
            },
            _ => todo!()
        };
        EmptyTileImage {
            inner,
            //pixel_fill,
        }
    }
}

use cgmath::Vector3;
use al_core::{
    Texture2DArray,
    image::{Image, format::{R16I, R32I, R8UI, R32F}},
};

impl Image for EmptyTileImage {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        self.inner.tex_sub_image_3d(textures, offset);
    }
}

use al_core::image::format::{ImageFormatType, RGB8U, RGBA8U};

//use super::TileArrayBuffer;
use crate::WebGlContext;
use std::rc::Rc;

/*use super::{ArrayF32, ArrayF64, ArrayI16, ArrayI32, ArrayU8};
fn create_black_tile(format: FormatImageType, width: i32, value: f32) -> TileArrayBufferImage {
    let _num_channels = format.get_num_channels() as i32;
    match format {
        FormatImageType::JPG => TileArrayBufferImage::U8(JPG::create_black_tile(width)),
        FormatImageType::PNG => TileArrayBufferImage::U8(PNG::create_black_tile(width)),
        FormatImageType::FITS(_fits) => match format.get_type() {
            WebGl2RenderingContext::FLOAT => {
                TileArrayBufferImage::F32(FITS::create_black_tile(width, value))
            }
            WebGl2RenderingContext::INT => {
                TileArrayBufferImage::I32(FITS::create_black_tile(width, value as i32))
            }
            WebGl2RenderingContext::SHORT => {
                TileArrayBufferImage::I16(FITS::create_black_tile(width, value as i16))
            }
            WebGl2RenderingContext::UNSIGNED_BYTE => {
                TileArrayBufferImage::U8(FITS::create_black_tile(width, value as u8))
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}*/

#[derive(Debug)]
pub struct HiPSConfig {
    pub root_url: String,
    // HiPS image format
    pub empty_image: EmptyTileImage,

    // The size of the texture images
    pub texture_size: i32,
    tile_size: i32,

    // Longitude reversed
    pub longitude_reversed: bool,
    // Delta depth i.e. log2(texture_size / tile_size)
    delta_depth: u8,
    // Num tiles per texture
    num_tiles_per_texture: usize,
    // Max depth of the current HiPS tiles
    max_depth_tile: u8,
    max_depth_texture: u8,
    num_textures_by_side_slice: i32,
    num_textures_by_slice: i32,
    num_slices: i32,
    num_textures: usize,

    // TODO: store this values in the ImageSurvey
    // These are proper to the survey (FITS one) and not
    // to a specific survey color
    pub scale: f32,
    pub offset: f32,
    pub blank: f32,

    pub tex_storing_integers: bool,
    pub tex_storing_fits: bool,
    pub tex_storing_unsigned_int: bool,

    pub size_tile_uv: f32,
    pub frame: CooSystem,
    format: ImageFormatType,
}

use al_api::coo_system::CooSystem;
use crate::math;
use crate::{HiPSProperties};
use al_api::hips::HiPSTileFormat;
use wasm_bindgen::JsValue;

impl HiPSConfig {
    /// Define a HiPS configuration
    ///
    /// # Arguments
    ///
    /// * `properties` - A description of the HiPS, its metadata, available formats  etc...
    /// * `img_format` - Image format wanted by the user
    pub fn new(properties: &HiPSProperties, img_format: HiPSTileFormat) -> Result<HiPSConfig, JsValue> {
        let root_url = properties.get_url();
        // Define the size of the 2d texture array depending on the
        // characterics of the client
        let num_textures_by_side_slice = 8;
        let num_textures_by_slice = num_textures_by_side_slice * num_textures_by_side_slice;
        let num_slices = 3;
        let num_textures = (num_textures_by_slice * num_slices) as usize;

        let max_depth_tile = properties.get_max_order();
        let tile_size = properties.get_tile_size();
        // Assert size is a power of two
        // Determine the size of the texture to copy
        // it cannot be > to 512x512px

        let fmt = properties.get_formats();
        let longitude_reversed = properties.longitude_reversed;
        let bitpix = properties.get_bitpix();
        let mut tex_storing_unsigned_int = false;
        let mut tex_storing_integers = false;

        let mut tex_storing_fits = false;
        
        
        if !properties.get_formats().contains(&img_format) {
            al_core::log(&format!("{:?} {:?}", properties.get_formats(), img_format));
            return Err(js_sys::Error::new("HiPS format not available").into());
        }

        let format = match img_format {
            HiPSTileFormat::FITS => {
                tex_storing_fits = true;
                // Check the bitpix to determine the internal format of the tiles
                if let Some(bitpix) = bitpix {
                    match bitpix {
                        #[cfg(feature = "webgl2")]
                        8 => {
                            tex_storing_unsigned_int = true;
                            Ok(ImageFormatType::R8UI)
                        }
                        #[cfg(feature = "webgl2")]
                        16 => {
                            tex_storing_integers = true;
                            Ok(ImageFormatType::R16I)
                        }
                        #[cfg(feature = "webgl2")]
                        32 => {
                            tex_storing_integers = true;
                            Ok(ImageFormatType::R32I)
                        }
                        -32 => {
                            tex_storing_integers = false;
                            Ok(ImageFormatType::R32F)
                        }
                        -64 => {
                            tex_storing_integers = false;
                            Err(JsValue::from_str("f64 FITS files not supported"))
                        }
                        _ => Err(JsValue::from_str("Fits tiles exists but the BITPIX is not correct in the property file"))
                    }
                } else {
                    Err(JsValue::from_str("Fits tiles exists but the BITPIX is not found"))
                }
            },
            HiPSTileFormat::PNG => Ok(ImageFormatType::RGBA8U),
            HiPSTileFormat::JPEG => Ok(ImageFormatType::RGB8U)
        }?;

        let empty_image = EmptyTileImage::new(tile_size, format);

        let texture_size = std::cmp::min(512, tile_size << max_depth_tile);
        let num_tile_per_side_texture = (texture_size / tile_size) as usize;

        let delta_depth = math::utils::log_2_unchecked(num_tile_per_side_texture) as u8;
        let num_tiles_per_texture = num_tile_per_side_texture * num_tile_per_side_texture;

        let max_depth_texture = max_depth_tile - delta_depth;
        let size_tile_uv = 1_f32 / ((8 << delta_depth) as f32);

        let frame = properties.get_frame();

        let hips_config = HiPSConfig {
            // HiPS name
            root_url,
            // Tile size & blank tile data
            empty_image,
            longitude_reversed,
            // Texture config
            // The size of the texture images
            texture_size,
            // Delta depth i.e. log2(texture_size / tile_size)
            delta_depth,
            // Num tiles per texture
            num_tiles_per_texture,
            // Max depth of the current HiPS tiles
            max_depth_texture,
            max_depth_tile,
            num_textures_by_side_slice,
            num_textures_by_slice,
            num_slices,
            num_textures,

            scale: 1.0,
            offset: 0.0,
            blank: -1.0, // by default, set it to -1

            tex_storing_fits,
            tex_storing_integers,
            tex_storing_unsigned_int,

            size_tile_uv,
            frame,
            format,
            tile_size,
        };

        Ok(hips_config)
    }

    #[inline]
    pub fn get_root_url(&self) -> &str {
        &self.root_url
    }

    #[inline]
    pub fn set_fits_metadata(&mut self, bscale: f32, bzero: f32, blank: f32) {
        self.scale = bscale;
        self.offset = bzero;
        self.blank = blank;
    }

    #[inline]
    pub fn delta_depth(&self) -> u8 {
        self.delta_depth
    }

    #[inline]
    pub fn num_tiles_per_texture(&self) -> usize {
        self.num_tiles_per_texture
    }

    #[inline]
    pub fn get_texture_size(&self) -> i32 {
        self.texture_size
    }

    #[inline]
    pub fn get_tile_size(&self) -> i32 {
        self.tile_size
    }
    /*
        #[inline]
        pub fn get_black_tile(&self) -> Rc<TileArrayBufferImage> {
            self.tile_config.get_black_tile()
        }
    */
    #[inline]
    pub fn get_max_depth(&self) -> u8 {
        self.max_depth_texture
    }

    #[inline]
    pub fn get_frame(&self) -> CooSystem {
        self.frame
    }

    /*#[inline]
    pub fn get_max_tile_depth(&self) -> u8 {
        self.max_depth_tile
    }*/

    #[inline]
    pub fn num_textures(&self) -> usize {
        self.num_textures
    }

    #[inline]
    pub fn num_textures_by_side_slice(&self) -> i32 {
        self.num_textures_by_side_slice
    }

    #[inline]
    pub fn num_textures_by_slice(&self) -> i32 {
        self.num_textures_by_slice
    }

    #[inline]
    pub fn num_slices(&self) -> i32 {
        self.num_slices
    }

    #[inline]
    pub fn get_format(&self) -> ImageFormatType {
        self.format
    }

    #[inline]
    pub fn get_default_image(&self) -> &EmptyTileImage {
        &self.empty_image
    }
}

use al_core::shader::{SendUniforms, ShaderBound};

impl SendUniforms for HiPSConfig {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        // Send max depth
        shader
            .attach_uniform("max_depth", &(self.max_depth_texture as i32))
            .attach_uniform("size_tile_uv", &self.size_tile_uv)
            .attach_uniform("tex_storing_fits", &self.tex_storing_fits)
            .attach_uniform("scale", &self.scale)
            .attach_uniform("offset", &self.offset)
            .attach_uniform("blank", &self.blank);
            //.attach_uniform("inversed_longitude", &self.longitude_reversed);

        shader
    }
}