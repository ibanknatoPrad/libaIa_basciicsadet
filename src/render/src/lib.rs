#[macro_use]
extern crate itertools_num;
extern crate serde_derive;
extern crate serde_json;
extern crate num;
extern crate task_async_executor;
extern crate console_error_panic_hook;
extern crate fitsreader;
use std::panic;

#[macro_use]
mod utils;

use wasm_bindgen::{prelude::*, JsCast};

mod shader;
mod shaders;
pub mod renderable;
mod finite_state_machine;
pub mod camera;
mod core;
mod math;
#[path = "../img/myfont.rs"]
mod myfont;
mod transfert_function;
mod projeted_grid;
//mod mouse_inertia;
mod event_manager;
mod color;
mod healpix_cell;
mod buffer;
mod rotation;
mod sphere_geometry;
mod cdshealpix;
mod async_task;
mod time;
mod image_fmt;
pub use image_fmt::FormatImageType;

use crate::{
    shader::{Shader, ShaderManager},
    renderable::{
        TextManager, Angle, ArcDeg,
        grid::ProjetedGrid,
        catalog::{Source, Manager},
        projection::{Aitoff, Orthographic, Mollweide, Gnomonic, AzimuthalEquidistant, Mercator, Projection},
    },
    camera::CameraViewPort,
    math::{LonLatT, LonLat},
    async_task::{TaskResult, TaskType},
    buffer::HiPSConfig,
};

use std::{
    rc::Rc,
    cell::RefCell,
    collections::HashSet
};

use cgmath::Vector4;

use async_task::TaskExecutor;
use web_sys::WebGl2RenderingContext;
use crate::{
    buffer::TileDownloader,
    renderable::image_survey::ImageSurveys
};
struct App {
    gl: WebGl2Context,

    shaders: ShaderManager,
    camera: CameraViewPort,

    downloader: TileDownloader,
    surveys: ImageSurveys,
    
    time_start_blending: Time,
    request_redraw: bool,
    rendering: bool,

    // The grid renderable
    //grid: ProjetedGrid,
    // Catalog manager
    manager: Manager,
    // Text example
    //text_manager: TextManager,

    // Finite State Machine declarations
    /*user_move_fsm: UserMoveSphere,
    user_zoom_fsm: UserZoom,
    move_fsm: MoveSphere,*/

    // Task executor
    exec: Rc<RefCell<TaskExecutor>>,
    resources: Resources,

    move_animation: Option<MoveAnimation>,
    tasks_finished: bool,
}

#[derive(Debug, Deserialize)]
pub struct Resources(HashMap<String, String>);

impl Resources {
    pub fn get_filename<'a>(&'a self, name: &str) -> Option<&String> {
        self.0.get(name)
    }
}

use cgmath::{Vector2, Vector3};
use futures::stream::StreamExt; // for `next`

use crate::shaders::Colormap;
use crate::rotation::Rotation;
struct MoveAnimation {
    start_anim_rot: Rotation<f32>,
    goal_anim_rot: Rotation<f32>,
    time_start_anim: Time,
    goal_pos: Vector3<f32>,
}

const BLEND_TILE_ANIM_DURATION: f32 = 500.0; // in ms
use crate::time::Time;
use crate::renderable::angle::ArcSec;
use crate::renderable::image_survey::ImageSurvey;
use crate::buffer::Tile;
use cgmath::InnerSpace;
use crate::renderable::image_survey::HiPS;
impl App {
    fn new(gl: &WebGl2Context, mut shaders: ShaderManager, resources: Resources) -> Result<Self, JsValue> {
        let gl = gl.clone();
        let exec = Rc::new(RefCell::new(TaskExecutor::new()));
        //gl.enable(WebGl2RenderingContext::BLEND);
        //gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE);

        gl.blend_func_separate(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);
        //gl.blend_func_separate(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);

        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);

        let sdss = SimpleHiPS {
            properties: HiPSProperties {
                url: String::from("http://alasky.u-strasbg.fr/SDSS/DR9/color"),

                maxOrder: 10,
                frame: Frame { label: String::from("J2000"), system: String::from("J2000") },
                tileSize: 512,
                format: HiPSFormat::Image {
                    format: String::from("jpeg")
                },
                minCutout: None,
                maxCutout: None,
            },
            color: HiPSColor::Color
        };

        log("shaders compiled");
        //panic!(format!("{:?}", aa));
        let camera = CameraViewPort::new::<Orthographic>(&gl);

        // The tile buffer responsible for the tile requests
        let downloader = TileDownloader::new();
        // The surveys storing the textures of the resolved tiles
        let mut surveys = ImageSurveys::new::<Orthographic>(&gl, &camera, &mut shaders);

        let (survey, color) = sdss.create(&gl, &camera, &surveys, exec.clone())?;
        surveys.add_simple_survey(survey, color, 0);

        let time_start_blending = Time::now();

        // Catalog definition
        let manager = Manager::new(&gl, &mut shaders, &camera, &resources);

        // Text 
        let font = myfont::FONT_CONFIG;
        //let text_manager = TextManager::new(&gl, font, &mut shaders);
        /*let _text = TextUponSphere::new(
            String::from("Aladin-Lite"),
            //&Vector2::new(300_f32, 100_f32),
            &Vector4::new(0.0, 1.0, 0.0, 1.0),
            &gl,
            &_text_manager,
            &shaders,
        );*/

        // Grid definition
        //let grid = ProjetedGrid::new::<Orthographic>(&gl, &camera, &mut shaders, &text_manager);

        // Finite State Machines definitions
        /*let user_move_fsm = UserMoveSphere::init();
        let user_zoom_fsm = UserZoom::init();
        let move_fsm = MoveSphere::init();*/

        // Variable storing the location to move to
        let move_animation = None;
        let tasks_finished = false;
        let request_redraw = false;
        let start_render_time = Time::now();
        let rendering = true;
        let app = App {
            gl,

            shaders,

            camera,

            downloader,
            surveys,

            time_start_blending,
            rendering,
            request_redraw,
            // The grid renderable
            //grid,
            // The catalog renderable
            manager,
            //text_manager,
            
            // Finite state machines,
            /*user_move_fsm,
            user_zoom_fsm,
            move_fsm,*/

            exec,
            resources,

            move_animation,

            tasks_finished,
        };

        Ok(app)
    } 

    fn look_for_new_tiles(&mut self) {
        // Move the views of the different active surveys
        self.surveys.refresh_views(&self.camera);

        // Loop over the surveys
        for (survey_id, survey) in self.surveys.iter_mut() {
            let already_available_cells = {
                let mut already_available_cells = HashSet::new();

                let textures = survey.get_textures();
                let view = survey.get_view();
    
                let cells_in_fov = view.get_cells();
    
                for cell in cells_in_fov.iter() {
                    let already_available = textures.contains_tile(cell);
                    let is_cell_new = view.is_new(cell);
    
                    if already_available {
                        // Remove and append the texture with an updated
                        // time_request
                        if is_cell_new {
                            // New cells are 
                            self.time_start_blending = Time::now();
                        }
                        already_available_cells.insert((*cell, is_cell_new));
                    } else {
                        // Submit the request to the buffer
                        let format = textures.config().format();
                        let root_url = survey_id.clone();
                        let tile = Tile {
                            root_url,
                            format,
                            cell: *cell
                        };

                        self.downloader.request_tile(tile);
                    }
                }

                already_available_cells
            };
            let textures = survey.get_textures_mut();

            for (cell, is_new_cell) in already_available_cells {
                textures.update_priority(&cell, is_new_cell);
            }

        }
    }

    // Run async tasks:
    // - parsing catalogs
    // - copying textures to GPU
    // Return true when a task is complete. This always lead
    // to a redraw of aladin lite
    fn run_tasks<P: Projection>(&mut self, dt: DeltaTime) -> Result<HashSet<Tile>, JsValue> {
        //crate::log(&format!("last frame duration (ms): {:?}", dt));
        let results = self.exec.borrow_mut().run(dt.0 * 0.3_f32);
        self.tasks_finished = !results.is_empty();

        // Retrieve back all the tiles that have been
        // copied to the GPU
        // This is important for the tile buffer to know which
        // requests can be reused to query more tiles
        let mut tiles_available = HashSet::new();
        for result in results {
            match result {
                TaskResult::TableParsed { name, sources} => {
                    log("CATALOG FINISHED PARSED");
                    self.manager.add_catalog::<P>(
                        name,
                        sources,
                        Colormap::BluePastelRed,
                        &mut self.shaders,
                        &self.camera,
                        &self.surveys.get_view().unwrap()
                    );
                },
                TaskResult::TileSentToGPU { tile } => {
                    tiles_available.insert(tile);
                }
            }
        }

        Ok(tiles_available)
    }

    fn update<P: Projection>(&mut self,
        dt: DeltaTime,
    ) -> Result<(), JsValue> {
        let available_tiles = self.run_tasks::<P>(dt)?;
        let is_there_new_available_tiles = !available_tiles.is_empty();

        // Run the FSMs
        //self.user_move_fsm.run::<P>(dt, &mut self.sphere, &mut self.manager, &mut self.grid, &mut self.camera, &events);
        //self.user_zoom_fsm.run::<P>(dt, &mut self.sphere, &mut self.manager, &mut self.grid, &mut self.camera, &events);
        //self.move_fsm.run::<P>(dt, &mut self.sphere, &mut self.manager, &mut self.grid, &mut self.camera, &events);

        // Check if there is an move animation to do
        if let Some(MoveAnimation {start_anim_rot, goal_anim_rot, time_start_anim, goal_pos} ) = self.move_animation {
            let t = (utils::get_current_time() - time_start_anim.as_millis())/1000_f32;
        
            // Undamped angular frequency of the oscillator
            // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
            //
            // In a damped harmonic oscillator system: w0 = sqrt(k / m)
            // where: 
            // * k is the stiffness of the ressort
            // * m is its mass
            let alpha = 1_f32 + (0_f32 - 1_f32) * (5_f32 * t + 1_f32) * ((-5_f32 * t).exp());
            let p = start_anim_rot.slerp(&goal_anim_rot, alpha);
    
            self.camera.set_rotation::<P>(&p);
            self.look_for_new_tiles();

            // Animation stop criteria
            let cursor_pos = self.camera.get_center().truncate();
            let err = math::ang_between_vect(&goal_pos, &cursor_pos);
            let thresh: Angle<f32> = ArcSec(2_f32).into();
            if err < thresh {
                self.move_animation = None;
            }
        }

        // Update the grid in consequence
        //self.grid.update_label_positions::<P>(&self.gl, &mut self.text_manager, &self.camera, &self.shaders);
        //self.text.update_from_camera::<P>(&self.camera);
        {
            // Newly available tiles must lead to
            if is_there_new_available_tiles {
                self.time_start_blending = Time::now();
            }

            // 1. Surveys must be aware of the new available tiles
            self.surveys.set_available_tiles(&available_tiles);
            // 2. Get the resolved tiles and push them to the image surveys
            let resolved_tiles = self.downloader.get_resolved_tiles(&available_tiles, &self.surveys);
            self.surveys.add_resolved_tiles(resolved_tiles);
            // 3. Try sending new tile requests after 
            self.downloader.try_sending_tile_requests();
        }

        // The rendering is done following these different situations:
        // - the camera has moved
        let has_camera_moved = self.camera.has_camera_moved();

        // - there is at least one tile in its blending phase
        let blending_anim_occuring = (Time::now().0 - self.time_start_blending.0) < BLEND_TILE_ANIM_DURATION;
        self.rendering = blending_anim_occuring | has_camera_moved | self.request_redraw;
        self.request_redraw = false;

        // Finally update the camera that reset the flag camera changed
        if has_camera_moved {
            self.manager.update::<P>(&self.camera, self.surveys.get_view().unwrap());
        }

        Ok(())
    }

    fn render<P: Projection>(&mut self, _enable_grid: bool) {
        if !self.rendering {
            return;
        }

        // Render the scene
        self.gl.clear_color(0.08, 0.08, 0.08, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);


        //self.gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE);
        self.surveys.draw::<P>(&self.camera, &mut self.shaders);
        self.gl.enable(WebGl2RenderingContext::BLEND);
        //self.gl.blend_func_separate(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);
        // Draw the catalog
        self.manager.draw::<P>(
            &self.gl,
            &mut self.shaders,
            &self.camera
        );

        // Reset the flags about the user action
        self.camera.reset();
        
        // Draw the grid
        /*self.grid.draw::<P>(
            &self.gl,
            shaders,
            camera,
            &self.text_manager
        );*/

        /*self.text_manager.draw(
            &self.gl,
            shaders,
            camera
        );*/
        self.gl.disable(WebGl2RenderingContext::BLEND);
    }

    fn set_simple_hips<P: Projection>(&mut self, hips: SimpleHiPS) -> Result<(), JsValue> {
        let (survey, color) = hips.create(&self.gl, &self.camera, &self.surveys, self.exec.clone())?;
        let textures = survey.get_textures();

        let format = textures.config().format();
        let root_url = survey.get_id().to_string();

        let new_survey = self.surveys.add_simple_survey(survey, color, 0);
        
        if new_survey {
            self.downloader.clear_requests();
            self.downloader.request_base_tiles(&root_url, &format);
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }
    fn set_overlay_simple_hips<P: Projection>(&mut self, hips: SimpleHiPS) -> Result<(), JsValue> {
        let (survey, color) = hips.create(&self.gl, &self.camera, &self.surveys, self.exec.clone())?;
        let textures = survey.get_textures();

        let format = textures.config().format();
        let root_url = survey.get_id().to_string();

        let new_survey = self.surveys.add_simple_survey(survey, color, 1);
        
        if new_survey {
            self.downloader.clear_requests();
            self.downloader.request_base_tiles(&root_url, &format);
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }

    fn set_composite_hips<P: Projection>(&mut self, hipses: CompositeHiPS) -> Result<(), JsValue> {
        let mut surveys = Vec::new();
        let mut colors = Vec::new();
        let mut survey_ids = Vec::new();
        let mut survey_formats = Vec::new();

        for hips in hipses.into_iter() {
            let (survey, color) = hips.create(&self.gl, &self.camera, &self.surveys, self.exec.clone())?;

            survey_ids.push(survey.get_id().to_string());
            survey_formats.push(survey.get_textures().config.format());

            surveys.push(survey);
            colors.push(color);
        }

        let new_survey_ids = self.surveys.add_composite_surveys(surveys, colors, 0);
        
        if !new_survey_ids.is_empty() {
            self.downloader.clear_requests();
            for id in new_survey_ids.iter() {
                let format = self.surveys.get(id).unwrap().get_textures().config.format();
                self.downloader.request_base_tiles(id, &format);
            }
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }

    fn set_overlay_composite_hips<P: Projection>(&mut self, hipses: CompositeHiPS) -> Result<(), JsValue> {
        let mut surveys = Vec::new();
        let mut colors = Vec::new();
        let mut survey_ids = Vec::new();
        let mut survey_formats = Vec::new();

        for hips in hipses.into_iter() {
            let (survey, color) = hips.create(&self.gl, &self.camera, &self.surveys, self.exec.clone())?;

            survey_ids.push(survey.get_id().to_string());
            survey_formats.push(survey.get_textures().config.format());

            surveys.push(survey);
            colors.push(color);
        }

        let new_survey_ids = self.surveys.add_composite_surveys(surveys, colors, 1);
        
        if !new_survey_ids.is_empty() {
            self.downloader.clear_requests();
            for id in new_survey_ids.iter() {
                let format = self.surveys.get(id).unwrap().get_textures().config.format();
                self.downloader.request_base_tiles(id, &format);
            }
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }

    fn set_overlay_opacity(&mut self, opacity: f32) -> Result<(), JsValue> {
        self.surveys.set_overlay_opacity(opacity);
        self.request_redraw = true;
        Ok(())
    }

    fn set_projection<P: Projection>(&mut self) {
        self.camera.set_projection::<P>();
        self.surveys.set_projection::<P>(&self.camera, &mut self.shaders);

        self.look_for_new_tiles();
        self.request_redraw = true;
    }

    fn set_longitude_reversed<P: Projection>(&mut self, reversed: bool) {
        /*if reversed {
            self.gl.cull_face(WebGl2RenderingContext::BACK);
        } else {
            self.gl.cull_face(WebGl2RenderingContext::FRONT);
        }*/

        self.camera.set_longitude_reversed(reversed);
        self.surveys.set_longitude_reversed::<P>(reversed, &self.camera, &mut self.shaders);

        self.look_for_new_tiles();

        self.request_redraw = true;
    }

    fn add_catalog(&mut self, name: String, table: JsValue) {
        let mut exec_ref = self.exec.borrow_mut();
        let table = table;
        exec_ref.spawner().spawn(TaskType::ParseTable, async {
            let mut stream = async_task::ParseTable::<[f32; 4]>::new(table);
            let mut results: Vec<Source> = vec![];
        
            while let Some(item) = stream.next().await {
                let item: &[f32] = item.as_ref();
                results.push(item.into());
            }
        
            TaskResult::TableParsed { name, sources: results }
        });
    }

    fn resize_window<P: Projection>(&mut self, width: f32, height: f32, _enable_grid: bool) {
        self.camera.set_screen_size::<P>(width, height);

        // Launch the new tile requests
        self.look_for_new_tiles();
        self.manager.set_kernel_size(&self.camera);
    }

    fn set_colormap(&mut self, name: String, colormap: Colormap) {
        self.manager.get_mut_catalog(&name)
            .unwrap()
            .set_colormap(colormap);
    }

    fn set_heatmap_opacity(&mut self, name: String, opacity: f32) {
        self.manager.get_mut_catalog(&name)
            .unwrap()
            .set_alpha(opacity);
    }

    fn set_kernel_strength<P: Projection>(&mut self, name: String, strength: f32) {
        self.manager.get_mut_catalog(&name)
            .unwrap()
            .set_strength(strength);
    }

    pub fn set_grid_color(&mut self, _red: f32, _green: f32, _blue: f32) {
        //self.grid.set_color(red, green, blue);
    }

    /*pub fn set_cutouts(&mut self, survey_url: &str, min_cutout: f32, max_cutout: f32) -> Result<(), String> {
        let survey = self.surveys.get(survey_url)
            .ok_or(format!("{} survey not found!", survey_url))?;
        survey.config_mut()
            .set_cutouts(min_cutout, max_cutout);

        Ok(())
    }

    pub fn set_transfer_func(&mut self, survey_url: &str, h: TransferFunction) -> Result<(), String> {
        let survey = self.surveys.get(survey_url)
            .ok_or(format!("{} survey not found!", survey_url))?;
        survey.config_mut()
            .set_transfer_function(h);

        Ok(())
    }

    pub fn set_fits_colormap(&mut self, survey_url: &str, colormap: Colormap) -> Result<(), String> {
        let survey = self.surveys.get(survey_url)
            .ok_or(format!("{} survey not found!", survey_url))?;
        survey.config_mut().set_fits_colormap(colormap);

        Ok(())
    }*/

    pub fn set_grid_opacity(&mut self, _alpha: f32) {
        //self.grid.set_alpha(alpha);
    }

    pub fn screen_to_world<P: Projection>(&self, pos: &Vector2<f32>) -> Result<LonLatT<f32>, String> {
        let model_pos = P::screen_to_model_space(pos, &self.camera).ok_or(format!("{:?} is out of projection", pos))?;
        Ok(model_pos.lonlat())
    }

    pub fn set_center<P: Projection>(&mut self, lonlat: &LonLatT<f32>) {
        let xyz: Vector4<f32> = lonlat.vector();
        let rot = Rotation::from_sky_position(&xyz);
        self.camera.set_rotation::<P>(&rot);
        self.look_for_new_tiles();

        // Stop the current animation if there is one
        self.move_animation = None;
    }

    pub fn start_moving_to<P: Projection>(&mut self, lonlat: &LonLatT<f32>) {
        // Get the XYZ cartesian position from the lonlat
        let cursor_pos = self.camera.get_center();
        let goal_pos: Vector4<f32> = lonlat.vector();

        // Convert these positions to rotations
        let start_anim_rot = *self.camera.get_rotation();
        let goal_anim_rot = Rotation::from_sky_position(&goal_pos);

        // Set the moving animation object
        self.move_animation = Some(MoveAnimation {
            time_start_anim: Time::now(),
            start_anim_rot,
            goal_anim_rot,
            goal_pos: goal_pos.truncate(),
        });
    }

    pub fn go_from_to<P: Projection>(&mut self, pos1: &LonLatT<f32>, pos2: &LonLatT<f32>) {
        let model2world = self.camera.get_m2w();

        let m1: Vector4<f32> = pos1.vector();
        let w1 = model2world * m1;
        let m2: Vector4<f32> = pos2.vector();
        let w2 = model2world * m2;

        let r = self.camera.get_rotation();

        let x = r.rotate(&w1).truncate();
        let y = r.rotate(&w2).truncate();
        if x != y {
            let axis = x.cross(y)
                .normalize();
            let d = math::ang_between_vect(&x, &y);
    
            self.camera.rotate::<P>(&(-axis), d);
            self.look_for_new_tiles();
        }

        // Stop the current animation if there is one
        self.move_animation = None;
    }
    
    pub fn set_fov<P: Projection>(&mut self, fov: &Angle<f32>) {
        // Change the camera rotation
        self.camera.set_aperture::<P>(*fov);
        self.look_for_new_tiles();
    }

    // Accessors
    fn get_center<P: Projection>(&self) -> LonLatT<f32> {
        //let center_pos = self.camera.compute_center_model_pos::<P>();
        self.camera.get_center().lonlat()
    }

    fn get_fov(&self) -> f32 {
        let deg: ArcDeg<f32> = self.camera.get_aperture().into();
        deg.0
    }
}

#[derive(Clone)]
pub struct WebGl2Context {
    inner: Rc<WebGl2RenderingContext>,
}

impl WebGl2Context {
    fn new() -> WebGl2Context {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let canvas = document.get_elements_by_class_name("aladin-imageCanvas").get_with_index(0).unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let context_options = js_sys::JSON::parse(&"{\"antialias\":false}").unwrap();
        let inner = Rc::new(
            canvas.get_context_with_context_options("webgl2", context_options.as_ref())
                .unwrap()
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap()
        );

        WebGl2Context {
            inner,
        }
    }
}

use std::ops::Deref;
impl Deref for WebGl2Context {
    type Target = WebGl2RenderingContext;

    fn deref(&self) -> &WebGl2RenderingContext {
        &self.inner
    }
}

enum ProjectionType {
    Aitoff,
    MollWeide,
    Arc,
    Mercator,
    Ortho,
    Gnomonic,
}

impl ProjectionType {
    fn set_projection(&mut self, app: &mut App, name: String) -> Result<(), JsValue> {
        match name.as_str() {
            "aitoff" => {
                app.set_projection::<Aitoff>();
                *self = ProjectionType::Aitoff;
                Ok(())
            },
            "orthographic" => {
                app.set_projection::<Orthographic>();
                *self = ProjectionType::Ortho;
                Ok(())
            },
            "mollweide" => {
                app.set_projection::<Mollweide>();
                *self = ProjectionType::MollWeide;
                Ok(())
            },
            "arc" => {
                app.set_projection::<AzimuthalEquidistant>();
                *self = ProjectionType::Arc;
                Ok(())
            },
            "gnomonic" => {
                app.set_projection::<Gnomonic>();
                *self = ProjectionType::Gnomonic;
                Ok(())
            },
            "mercator" => {
                app.set_projection::<Mercator>();
                *self = ProjectionType::Mercator;
                Ok(())
            },
            _ => Err(format!("{} is not a valid projection name. aitoff, arc, orthographic, gnomonic, mollweide and mercator are accepted", name).into())
        }
    }

    fn set_longitude_reversed(&mut self, app: &mut App, reversed: bool) {
        match self {
            ProjectionType::Aitoff => app.set_longitude_reversed::<Aitoff>(reversed),
            ProjectionType::MollWeide => app.set_longitude_reversed::<Mollweide>(reversed),
            ProjectionType::Ortho => app.set_longitude_reversed::<Orthographic>(reversed),
            ProjectionType::Arc => app.set_longitude_reversed::<AzimuthalEquidistant>(reversed),
            ProjectionType::Gnomonic => app.set_longitude_reversed::<Gnomonic>(reversed),
            ProjectionType::Mercator => app.set_longitude_reversed::<Mercator>(reversed),
        };
    }

    fn set_colormap(&self, app: &mut App, name: String, colormap: Colormap) {
        match self {
            ProjectionType::Aitoff => app.set_colormap(name, colormap),
            ProjectionType::MollWeide => app.set_colormap(name, colormap),
            ProjectionType::Ortho => app.set_colormap(name, colormap),
            ProjectionType::Arc => app.set_colormap(name, colormap),
            ProjectionType::Gnomonic => app.set_colormap(name, colormap),
            ProjectionType::Mercator => app.set_colormap(name, colormap),
        };
    }

    fn screen_to_world(&self, app: &App, pos: &Vector2<f32>) -> Result<LonLatT<f32>, String> {
        match self {
            ProjectionType::Aitoff => app.screen_to_world::<Aitoff>(pos),
            ProjectionType::MollWeide => app.screen_to_world::<Mollweide>(pos),
            ProjectionType::Ortho => app.screen_to_world::<Orthographic>(pos),
            ProjectionType::Arc => app.screen_to_world::<AzimuthalEquidistant>(pos),
            ProjectionType::Gnomonic => app.screen_to_world::<Gnomonic>(pos),
            ProjectionType::Mercator => app.screen_to_world::<Mercator>(pos),
        }
    }

    fn go_from_to(&self, app: &mut App, pos1: &LonLatT<f32>, pos2: &LonLatT<f32>) {
        match self {
            ProjectionType::Aitoff => app.go_from_to::<Aitoff>(pos1, pos2),
            ProjectionType::MollWeide => app.go_from_to::<Mollweide>(pos1, pos2),
            ProjectionType::Ortho => app.go_from_to::<Orthographic>(pos1, pos2),
            ProjectionType::Arc => app.go_from_to::<AzimuthalEquidistant>(pos1, pos2),
            ProjectionType::Gnomonic => app.go_from_to::<Gnomonic>(pos1, pos2),
            ProjectionType::Mercator => app.go_from_to::<Mercator>(pos1, pos2),
        }
    }

    fn update(&mut self, app: &mut App, dt: DeltaTime) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.update::<Aitoff>(dt),
            ProjectionType::MollWeide => app.update::<Mollweide>(dt),
            ProjectionType::Ortho => app.update::<Orthographic>(dt),
            ProjectionType::Arc => app.update::<AzimuthalEquidistant>(dt),
            ProjectionType::Gnomonic => app.update::<Gnomonic>(dt),
            ProjectionType::Mercator => app.update::<Mercator>(dt),
        }
    }

    fn render(&mut self, app: &mut App, enable_grid: bool) {
        match self {
            ProjectionType::Aitoff => app.render::<Aitoff>(enable_grid),
            ProjectionType::MollWeide => app.render::<Mollweide>(enable_grid),
            ProjectionType::Ortho => app.render::<Orthographic>(enable_grid),
            ProjectionType::Arc => app.render::<AzimuthalEquidistant>(enable_grid),
            ProjectionType::Gnomonic => app.render::<Gnomonic>(enable_grid),
            ProjectionType::Mercator => app.render::<Mercator>(enable_grid),
        };
    }

    pub fn add_catalog(&mut self, app: &mut App, name: String, table: JsValue) {
        match self {
            ProjectionType::Aitoff => app.add_catalog(name, table),
            ProjectionType::MollWeide => app.add_catalog(name, table),
            ProjectionType::Ortho => app.add_catalog(name, table),
            ProjectionType::Arc => app.add_catalog(name, table),
            ProjectionType::Gnomonic => app.add_catalog(name, table),
            ProjectionType::Mercator => app.add_catalog(name, table),
        };
    }

    pub fn set_simple_hips(&mut self, app: &mut App, hips: SimpleHiPS) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_simple_hips::<Aitoff>(hips),
            ProjectionType::MollWeide => app.set_simple_hips::<Mollweide>(hips),
            ProjectionType::Ortho => app.set_simple_hips::<Orthographic>(hips),
            ProjectionType::Arc => app.set_simple_hips::<AzimuthalEquidistant>(hips),
            ProjectionType::Gnomonic => app.set_simple_hips::<Gnomonic>(hips),
            ProjectionType::Mercator => app.set_simple_hips::<Mercator>(hips),
        }
    }

    pub fn set_composite_hips(&mut self, app: &mut App, hips: CompositeHiPS) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_composite_hips::<Aitoff>(hips),
            ProjectionType::MollWeide => app.set_composite_hips::<Mollweide>(hips),
            ProjectionType::Ortho => app.set_composite_hips::<Orthographic>(hips),
            ProjectionType::Arc => app.set_composite_hips::<AzimuthalEquidistant>(hips),
            ProjectionType::Gnomonic => app.set_composite_hips::<Gnomonic>(hips),
            ProjectionType::Mercator => app.set_composite_hips::<Mercator>(hips),
        }
    }

    pub fn set_overlay_simple_hips(&mut self, app: &mut App, hips: SimpleHiPS) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_overlay_simple_hips::<Aitoff>(hips),
            ProjectionType::MollWeide => app.set_overlay_simple_hips::<Mollweide>(hips),
            ProjectionType::Ortho => app.set_overlay_simple_hips::<Orthographic>(hips),
            ProjectionType::Arc => app.set_overlay_simple_hips::<AzimuthalEquidistant>(hips),
            ProjectionType::Gnomonic => app.set_overlay_simple_hips::<Gnomonic>(hips),
            ProjectionType::Mercator => app.set_overlay_simple_hips::<Mercator>(hips),
        }
    }

    pub fn set_overlay_composite_hips(&mut self, app: &mut App, hips: CompositeHiPS) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_overlay_composite_hips::<Aitoff>(hips),
            ProjectionType::MollWeide => app.set_overlay_composite_hips::<Mollweide>(hips),
            ProjectionType::Ortho => app.set_overlay_composite_hips::<Orthographic>(hips),
            ProjectionType::Arc => app.set_overlay_composite_hips::<AzimuthalEquidistant>(hips),
            ProjectionType::Gnomonic => app.set_overlay_composite_hips::<Gnomonic>(hips),
            ProjectionType::Mercator => app.set_overlay_composite_hips::<Mercator>(hips),
        }
    }
    pub fn set_overlay_opacity(&mut self, app: &mut App, opacity: f32) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_overlay_opacity(opacity),
            ProjectionType::MollWeide => app.set_overlay_opacity(opacity),
            ProjectionType::Ortho => app.set_overlay_opacity(opacity),
            ProjectionType::Arc => app.set_overlay_opacity(opacity),
            ProjectionType::Gnomonic => app.set_overlay_opacity(opacity),
            ProjectionType::Mercator => app.set_overlay_opacity(opacity),
        }
    }

    pub fn resize(&mut self, app: &mut App, width: f32, height: f32, enable_grid: bool) {       
        match self {
            ProjectionType::Aitoff => app.resize_window::<Aitoff>(width, height, enable_grid),
            ProjectionType::MollWeide => app.resize_window::<Mollweide>(width, height, enable_grid),
            ProjectionType::Ortho => app.resize_window::<Orthographic>(width, height, enable_grid),
            ProjectionType::Arc => app.resize_window::<AzimuthalEquidistant>(width, height, enable_grid),
            ProjectionType::Gnomonic => app.resize_window::<Gnomonic>(width, height, enable_grid),
            ProjectionType::Mercator => app.resize_window::<Mercator>(width, height, enable_grid),
        }; 
    }

    pub fn set_kernel_strength(&mut self, app: &mut App, name: String, strength: f32) {        
        match self {
            ProjectionType::Aitoff => app.set_kernel_strength::<Aitoff>(name, strength),
            ProjectionType::MollWeide => app.set_kernel_strength::<Mollweide>(name, strength),
            ProjectionType::Ortho => app.set_kernel_strength::<Orthographic>(name, strength),
            ProjectionType::Arc => app.set_kernel_strength::<AzimuthalEquidistant>(name, strength),
            ProjectionType::Gnomonic => app.set_kernel_strength::<Gnomonic>(name, strength),
            ProjectionType::Mercator => app.set_kernel_strength::<Mercator>(name, strength),
        };
    }

    pub fn set_heatmap_opacity(&mut self, app: &mut App, name: String, opacity: f32) {       
        match self {
            ProjectionType::Aitoff => app.set_heatmap_opacity(name, opacity),
            ProjectionType::MollWeide => app.set_heatmap_opacity(name, opacity),
            ProjectionType::Ortho => app.set_heatmap_opacity(name, opacity),
            ProjectionType::Arc => app.set_heatmap_opacity(name, opacity),
            ProjectionType::Gnomonic => app.set_heatmap_opacity(name, opacity),
            ProjectionType::Mercator => app.set_heatmap_opacity(name, opacity),
        }; 
    }

    pub fn set_center(&mut self, app: &mut App, lonlat: LonLatT<f32>) {
        match self {
            ProjectionType::Aitoff => app.set_center::<Aitoff>(&lonlat),
            ProjectionType::MollWeide => app.set_center::<Mollweide>(&lonlat),
            ProjectionType::Ortho => app.set_center::<Orthographic>(&lonlat),
            ProjectionType::Arc => app.set_center::<AzimuthalEquidistant>(&lonlat),
            ProjectionType::Gnomonic => app.set_center::<Gnomonic>(&lonlat),
            ProjectionType::Mercator => app.set_center::<Mercator>(&lonlat),
        };
    }

    pub fn start_moving_to(&mut self, app: &mut App, lonlat: LonLatT<f32>) {
        match self {
            ProjectionType::Aitoff => app.start_moving_to::<Aitoff>(&lonlat),
            ProjectionType::MollWeide => app.start_moving_to::<Mollweide>(&lonlat),
            ProjectionType::Ortho => app.start_moving_to::<Orthographic>(&lonlat),
            ProjectionType::Arc => app.start_moving_to::<AzimuthalEquidistant>(&lonlat),
            ProjectionType::Gnomonic => app.start_moving_to::<Gnomonic>(&lonlat),
            ProjectionType::Mercator => app.start_moving_to::<Mercator>(&lonlat),
        };
    }

    pub fn set_fov(&mut self, app: &mut App, fov: Angle<f32>) {
        match self {
            ProjectionType::Aitoff => app.set_fov::<Aitoff>(&fov),
            ProjectionType::MollWeide => app.set_fov::<Mollweide>(&fov),
            ProjectionType::Ortho => app.set_fov::<Orthographic>(&fov),
            ProjectionType::Arc => app.set_fov::<AzimuthalEquidistant>(&fov),
            ProjectionType::Gnomonic => app.set_fov::<Gnomonic>(&fov),
            ProjectionType::Mercator => app.set_fov::<Mercator>(&fov),
        };
    }

    pub fn get_center(&self, app: &App) -> LonLatT<f32> {
        match self {
            ProjectionType::Aitoff => app.get_center::<Aitoff>(),
            ProjectionType::MollWeide => app.get_center::<Mollweide>(),
            ProjectionType::Ortho => app.get_center::<Orthographic>(),
            ProjectionType::Arc => app.get_center::<AzimuthalEquidistant>(),
            ProjectionType::Gnomonic => app.get_center::<Gnomonic>(),
            ProjectionType::Mercator => app.get_center::<Mercator>(),
        }
    }

    pub fn set_grid_color(&mut self, app: &mut App, red: f32, green: f32, blue: f32) {
        match self {
            ProjectionType::Aitoff => app.set_grid_color(red, green, blue),
            ProjectionType::MollWeide => app.set_grid_color(red, green, blue),
            ProjectionType::Ortho => app.set_grid_color(red, green, blue),
            ProjectionType::Arc => app.set_grid_color(red, green, blue),
            ProjectionType::Gnomonic => app.set_grid_color(red, green, blue),
            ProjectionType::Mercator => app.set_grid_color(red, green, blue),
        };
    }

    /*pub fn set_cutouts(&mut self, app: &mut App, min_cutout: f32, max_cutout: f32) -> Result<(), String> {
        match self {
            ProjectionType::Aitoff => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::MollWeide => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Ortho => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Arc => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Mercator => app.set_cutouts(min_cutout, max_cutout),
        };
    }

    pub fn set_transfer_func(&mut self, app: &mut App, h: TransferFunction) -> Result<(), String> {
        match self {
            ProjectionType::Aitoff => app.set_transfer_func(h),
            ProjectionType::MollWeide => app.set_transfer_func(h),
            ProjectionType::Ortho => app.set_transfer_func(h),
            ProjectionType::Arc => app.set_transfer_func(h),
            ProjectionType::Mercator => app.set_transfer_func(h),
        };
    }

    pub fn set_fits_colormap(&mut self, app: &mut App, colormap: Colormap) -> Result<(), String> {
        match self {
            ProjectionType::Aitoff => app.set_fits_colormap(colormap),
            ProjectionType::MollWeide => app.set_fits_colormap(colormap),
            ProjectionType::Ortho => app.set_fits_colormap(colormap),
            ProjectionType::Arc => app.set_fits_colormap(colormap),
            ProjectionType::Mercator => app.set_fits_colormap(colormap),
        };
    }*/

    pub fn set_grid_opacity(&mut self, app: &mut App, alpha: f32) {
        match self {
            ProjectionType::Aitoff => app.set_grid_opacity(alpha),
            ProjectionType::MollWeide => app.set_grid_opacity(alpha),
            ProjectionType::Ortho => app.set_grid_opacity(alpha),
            ProjectionType::Arc => app.set_grid_opacity(alpha),
            ProjectionType::Gnomonic => app.set_grid_opacity(alpha),
            ProjectionType::Mercator => app.set_grid_opacity(alpha),
        };
    }
}

use crate::event_manager::{
 EventManager,
 MoveToLocation,
 SetCenterLocation,
 StartInertia,
 SetFieldOfView,
 ZoomToLocation
};
use crate::time::DeltaTime;
#[wasm_bindgen]
pub struct WebClient {
    gl: WebGl2Context,
    // The app
    app: App,
    projection: ProjectionType,

    // The time between the previous and the current
    // frame
    dt: DeltaTime,

    // Some booleans for enabling/desabling
    // specific computations
    enable_inertia: bool,
    enable_grid: bool,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

use serde::Deserialize; 
#[derive(Debug, Deserialize)]
pub struct FileSrc {
    pub id: String,
    pub content: String,
}
use crate::transfert_function::TransferFunction;
use std::collections::HashMap;
use crate::healpix_cell::HEALPixCell;


#[derive(Deserialize)]
#[derive(Debug)]
pub struct Frame {
    pub label: String,
    pub system: String,
}
use js_sys::JsString;
#[derive(Deserialize)]
#[derive(Debug)]
pub struct HiPSProperties {
    pub url: String,

    pub maxOrder: u8,
    pub frame: Frame,
    pub tileSize: i32,
    pub minCutout: Option<f32>,
    pub maxCutout: Option<f32>,
    pub format: HiPSFormat,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub enum HiPSFormat {
    FITSImage {
        bitpix: i32
    },
    Image {
        format: String,
    }
}

#[derive(Deserialize)]
#[derive(Debug)]
pub enum HiPSColor {
    Grayscale2Colormap {
        colormap: String,
        transfer: String,
    },
    Grayscale2Color {
        color: [f32; 3],
        transfer: String,
        k: f32 // contribution of the component
    },
    Color
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct SimpleHiPS {
    properties: HiPSProperties,
    color: HiPSColor,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct CompositeHiPS {
    hipses: Vec<SimpleHiPS>
}
use std::iter::IntoIterator;
impl IntoIterator for CompositeHiPS {
    type Item = SimpleHiPS;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.hipses.into_iter()
    }
}

#[wasm_bindgen]
impl WebClient {
    /// Create a new web client
    #[wasm_bindgen(constructor)]
    pub fn new(shaders: &JsValue, resources: &JsValue) -> Result<WebClient, JsValue> {

        let shaders = shaders.into_serde::<Vec<FileSrc>>().unwrap();
        let resources = resources.into_serde::<Resources>().unwrap();
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let gl = WebGl2Context::new();

        let shaders = ShaderManager::new(&gl, shaders).unwrap();
        let app = App::new(&gl, shaders, resources)?;

        //let appconfig = AppConfig::Ortho(app);
        let dt = DeltaTime::zero();
        let enable_inertia = false;
        let enable_grid = true;
        let projection = ProjectionType::Ortho;

        let webclient = WebClient {
            gl,
            app,
            projection,

            dt,
            enable_inertia,
            enable_grid,
        };

        Ok(webclient)
    }

    /// Main update method
    pub fn update(&mut self, dt: f32) -> Result<(), JsValue> {
        // dt refers to the time taking (in ms) rendering the previous frame
        self.dt = DeltaTime::from_millis(dt);

        // Update the application and get back the
        // world coordinates of the center of projection in (ra, dec)
        self.projection.update(
            &mut self.app,
            // Time of the previous frame rendering 
            self.dt,
        )?;

        Ok(())
    }
    
    /// Update our WebGL Water application. `index.html` will call this function in order
    /// to begin rendering.
    pub fn render(&mut self, min_value: f32, max_value: f32) -> Result<(), JsValue> {
        self.projection.render(&mut self.app, self.enable_grid);

        Ok(())
    }

    /// Change the current projection of the HiPS
    #[wasm_bindgen(js_name = setProjection)]
    pub fn set_projection(&mut self, name: String) -> Result<(), JsValue> {
        self.projection.set_projection(&mut self.app, name)?;

        Ok(())
    }

    /// Change the current projection of the HiPS
    #[wasm_bindgen(js_name = setLongitudeReversed)]
    pub fn set_longitude_reversed(&mut self, reversed: bool) -> Result<(), JsValue> {
        self.projection.set_longitude_reversed(&mut self.app, reversed);

        Ok(())
    }

    /// Enable mouse inertia
    pub fn enable_inertia(&mut self) -> Result<(), JsValue> {
        self.enable_inertia = true;

        Ok(())
    }
    /// Disable mouse inertia
    pub fn disable_inertia(&mut self) -> Result<(), JsValue> {
        self.enable_inertia = false;

        Ok(())
    }

    /// Enable equatorial grid
    pub fn enable_equatorial_grid(&mut self) -> Result<(), JsValue> {
        self.enable_grid = true;

        //self.projection.enable_grid(&mut self.app);

        Ok(())
    }

    /// Disable equatorial grid
    pub fn disable_equatorial_grid(&mut self) -> Result<(), JsValue> {
        self.enable_grid = false;

        Ok(())
    }
    
    /// Change grid color
    pub fn set_grid_color(&mut self, red: f32, green: f32, blue: f32) -> Result<(), JsValue> {
        self.projection.set_grid_color(&mut self.app, red, green, blue);

        Ok(())
    }

    /// Change grid opacity
    pub fn set_grid_opacity(&mut self, alpha: f32) -> Result<(), JsValue> {
        self.projection.set_grid_opacity(&mut self.app, alpha);

        Ok(())
    }
    
    /*#[wasm_bindgen(js_name = setCutouts)]
    pub fn set_cutouts(&mut self, min_cutout: f32, max_cutout: f32) -> Result<(), JsValue> {
        self.projection.set_cutouts(&mut self.app, min_cutout, max_cutout).map_err(|e| e.into())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setFitsColormap)]
    pub fn set_fits_colormap(&mut self, colormap: String) -> Result<(), JsValue> {
        let colormap = Colormap::new(&colormap);
        self.projection.set_fits_colormap(&mut self.app, colormap).map_err(|e| e.into())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setTransferFunction)]
    pub fn set_transfer_func(&mut self, id: String) -> Result<(), JsValue> {
        let h = TransferFunction::new(&id);
        self.projection.set_transfer_func(&mut self.app, h).map_err(|e| e.into())?;

        Ok(())
    }*/

    // Set primary image survey
    #[wasm_bindgen(js_name = setSimpleHiPS)]
    pub fn set_simple_hips(&mut self, hips: JsValue) -> Result<(), JsValue> {

        let hips: SimpleHiPS = hips.into_serde().map_err(|e| e.to_string())?;
        crate::log(&format!("simple HiPS: {:?}", hips));

        self.projection.set_simple_hips(&mut self.app, hips)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setCompositeHiPS)]
    pub fn set_composite_hips(&mut self, hips: JsValue) -> Result<(), JsValue> {
        let hips: CompositeHiPS = hips.into_serde().map_err(|e| e.to_string())?;
        crate::log(&format!("Composite HiPS: {:?}", hips));

        self.projection.set_composite_hips(&mut self.app, hips)?;

        Ok(())
    }

    // Set an overlay HiPS
    #[wasm_bindgen(js_name = setOverlaySimpleHiPS)]
    pub fn set_overlay_simple_hips(&mut self, hips: JsValue) -> Result<(), JsValue> {
        let hips: SimpleHiPS = hips.into_serde().map_err(|e| e.to_string())?;
        crate::log(&format!("simple HiPS: {:?}", hips));

        self.projection.set_overlay_simple_hips(&mut self.app, hips)?;

        Ok(())
    }
    
    #[wasm_bindgen(js_name = setOverlayCompositeHiPS)]
    pub fn set_overlay_composite_hips(&mut self, hipses: JsValue) -> Result<(), JsValue> {
        let hipses: CompositeHiPS = hipses.into_serde().map_err(|e| e.to_string())?;
        crate::log(&format!("Composite HiPS: {:?}", hipses));

        self.projection.set_overlay_composite_hips(&mut self.app, hipses)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setOverlayOpacity)]
    pub fn set_overlay_opacity(&mut self, opacity: f32) -> Result<(), JsValue> {
        self.projection.set_overlay_opacity(&mut self.app, opacity)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = screenToWorld)]
    pub fn screen_to_world(&self, pos_x: f32, pos_y: f32) -> Result<Box<[f32]>, JsValue> {
        let lonlat = self.projection.screen_to_world(&self.app, &Vector2::new(pos_x, pos_y))?;

        let lon_deg: ArcDeg<f32> = lonlat.lon().into();
        let lat_deg: ArcDeg<f32> = lonlat.lat().into();

        Ok(Box::new([lon_deg.0, lat_deg.0]))
    }

    #[wasm_bindgen(js_name = getFieldOfView)]
    pub fn get_fov(&self) -> Result<f32, JsValue> {
        let fov = self.app.get_fov();
        Ok(fov)
    }

    /// Set directly the field of view (for pinch zooming)
    #[wasm_bindgen(js_name = setFieldOfView)]
    pub fn set_fov(&mut self, fov: f32) -> Result<(), JsValue> {
        self.projection.set_fov(&mut self.app, ArcDeg(fov).into());
        // Tell the finite state machines the fov has manually been changed
        //self.events.enable::<SetFieldOfView>(());

        Ok(())
    }
    
    #[wasm_bindgen(js_name = setClipZoomFactor)]
    pub fn set_clip_zoom_factor(&mut self, clip_zoom_factor: f32) -> Result<(), JsValue> {
        //self.projection.set_clip_zoom_factor(&mut self.app, fov);


        Ok(())
    }

    /// Set directly the center position
    #[wasm_bindgen(js_name = getCenter)]
    pub fn get_center(&self) -> Result<Box<[f32]>, JsValue> {
        let center = self.projection.get_center(&self.app);

        let lon_deg: ArcDeg<f32> = center.lon().into();
        let lat_deg: ArcDeg<f32> = center.lat().into();

        Ok(Box::new([lon_deg.0, lat_deg.0]))
    }

    /*#[wasm_bindgen(js_name = startInertia)]
    pub fn start_inertia(&mut self) -> Result<(), JsValue> {
        //self.projection.set_center(&mut self.app, ArcDeg(lon).into(), ArcDeg(lat).into());
        // Tell the finite state machines the center has manually been changed
        self.events.enable::<StartInertia>(());

        Ok(())
    }*/
/*
    /// Initiate a finite state machine that will zoom until a fov is reached
    /// while moving to a specific location
    #[wasm_bindgen(js_name = zoomToLocation)]
    pub fn set_zoom(&mut self, lon: f32, lat: f32, fov: f32) -> Result<(), JsValue> {
        self.events.enable::<ZoomToLocation>(
            (
                LonLatT::new(
                    ArcDeg(lon).into(),
                    ArcDeg(lat).into()
                ),
                ArcDeg(fov).into()
            )
        );

        Ok(())
    }*/
    /// Initiate a finite state machine that will move to a specific location
    #[wasm_bindgen(js_name = moveToLocation)]
    pub fn start_moving_to(&mut self, lon: f32, lat: f32) -> Result<(), JsValue> {
        // Enable the MouseLeftButtonReleased event
        let location = LonLatT::new(
            ArcDeg(lon).into(),
            ArcDeg(lat).into()
        );
        self.projection.start_moving_to(&mut self.app, location);

        /*self.events.enable::<MoveToLocation>(
            LonLatT::new(
                ArcDeg(lon).into(),
                ArcDeg(lat).into()
            )
        );*/

        Ok(())
    }
    /// Set directly the center position
    #[wasm_bindgen(js_name = setCenter)]
    pub fn set_center(&mut self, lon: f32, lat: f32) -> Result<(), JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());
        self.projection.set_center(&mut self.app, lonlat);

        Ok(())
    }

    #[wasm_bindgen(js_name = goFromTo)]
    pub fn go_from_to(&mut self, lon1: f32, lat1: f32, lon2: f32, lat2: f32) -> Result<(), JsValue> {
        let pos1 = LonLatT::new(
            ArcDeg(lon1).into(),
            ArcDeg(lat1).into()
        );
        let pos2 = LonLatT::new(
            ArcDeg(lon2).into(),
            ArcDeg(lat2).into()
        );
        self.projection.go_from_to(&mut self.app, &pos1, &pos2);

        Ok(())
    }

    /// CATALOG INTERFACE METHODS
    /// Add new catalog
    pub fn add_catalog(&mut self, name_catalog: String, data: JsValue) -> Result<(), JsValue> {
        self.projection.add_catalog(&mut self.app, name_catalog, data);

        Ok(())
    }

    /// Set the kernel strength
    pub fn set_kernel_strength(&mut self, name_catalog: String, strength: f32) -> Result<(), JsValue> {
        self.projection.set_kernel_strength(&mut self.app, name_catalog, strength);

        Ok(())
    }

    /// Set the heatmap global opacity
    pub fn set_heatmap_opacity(&mut self, name_catalog: String, opacity: f32) -> Result<(), JsValue> {
        self.projection.set_heatmap_opacity(&mut self.app, name_catalog, opacity);

        Ok(())
    }

    pub fn set_colormap(&mut self, name_catalog: String, name_colormap: String) -> Result<(), JsValue> {
        let colormap = match name_colormap.as_str() {
            "BluePastelRed" => Colormap::BluePastelRed,
            "IDL_CB_BrBG" => Colormap::IDLCBBrBG,
            "IDL_CB_YIGnBu" => Colormap::IDLCBYIGnBu,
            "IDL_CB_GnBu" => Colormap::IDLCBGnBu,
            "Red_Temperature" => Colormap::RedTemperature,
            "Black_White_Linear" => Colormap::BlackWhiteLinear,
            _ => panic!("{:?} colormap not recognized!", name_colormap)
        };
        self.projection.set_colormap(&mut self.app, name_catalog, colormap);

        Ok(())
    }

    /// Resize the window
    pub fn resize(&mut self, width: f32, height: f32) -> Result<(), JsValue> {
        self.projection.resize(&mut self.app, width, height, self.enable_grid);

        Ok(())
    }
}