use std::collections::HashSet;

use sdl2::{Sdl, mixer::{Sdl2MixerContext, DEFAULT_CHANNELS, Music, Chunk, AUDIO_S16LSB}, image::{Sdl2ImageContext, LoadTexture}, VideoSubsystem, render::{Canvas, Texture, TextureCreator, BlendMode}, video::{Window, WindowContext}, EventPump, event::Event, keyboard::Keycode, rect::{Rect, Point}, pixels::Color};
use spin_sleep::LoopHelper;

#[allow(dead_code)]
pub struct Init
{
    sdl: Sdl,
    sdl_mixer: Sdl2MixerContext,
    sdl_image: Sdl2ImageContext
}

impl Init
{
    pub fn new() -> Init
    {
        let context = sdl2::init().expect("Failed to init SDL!");

        //Init SDL_mixer context
        sdl2::mixer::open_audio(44_100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1024).expect("Failed to open audio!");
        let mixer_context = sdl2::mixer::init(sdl2::mixer::InitFlag::OGG).expect("Failed to init SDL_mixer!");
        sdl2::mixer::allocate_channels(6);

        //Init SDL_image context
        let image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).expect("Failed to init SDL_image!");

        Init { sdl: context, sdl_mixer: mixer_context, sdl_image: image_context }
    }

    pub fn context(&self) -> &Sdl
    {
        &self.sdl
    }
}

pub struct Video
{
    video: VideoSubsystem,
    canvas: Canvas<Window>
}

#[allow(dead_code)]
impl Video
{
    pub fn new(ctx: &Sdl, w: u32, h: u32) -> Video
    {
        let video = ctx.video().expect("Failed to create VideoSubsystem!");

        //Create window
        let window = video.window("Game", w, h).vulkan().resizable().position_centered().build().expect("Failed to open window!");

        //Try to create accelerated renderer
        let mut canvas = window.into_canvas().accelerated().build().expect("Failed to create renderer!");
        canvas.set_logical_size(w, h).expect("Failed to set logical size!");

        Video { video: video, canvas: canvas }
    }

    pub fn canvas(&self) -> &Canvas<Window>
    {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut Canvas<Window>
    {
        &mut self.canvas
    }

    pub fn video(&self) -> &VideoSubsystem
    {
        &self.video
    }

    pub fn video_mut(&mut self) -> &mut VideoSubsystem
    {
        &mut self.video
    }

}

pub struct Audio;

#[allow(dead_code)]
impl Audio
{
    pub fn new() -> Audio
    {
        Audio {}
    }

    pub fn load_music(&self, path: &str) -> Music
    {
        Music::from_file(path).expect("Failed to load music file!")
    }

    pub fn load_sound(&self, path: &str) -> Chunk
    {
        Chunk::from_file(path).expect("Failed to load WAV file!")
    }
}

pub struct EventLoop
{
    pub delta: f64,

    loop_helper: LoopHelper,
    event_pump: EventPump,
    keys: HashSet<Keycode>,
    prev_keys: HashSet<Keycode>
}

#[allow(dead_code)]
impl EventLoop 
{
    pub fn new(ctx: &Sdl, target_fps: u32) -> EventLoop
    {
        let loop_helper = LoopHelper::builder().build_with_target_rate(target_fps);
        let event_pump = ctx.event_pump().unwrap();
        EventLoop { loop_helper, event_pump, keys: HashSet::new(), prev_keys: HashSet::new(), delta: 0.0 }
    }

    pub fn loop_start(&mut self) -> bool
    {
        self.delta = self.loop_helper.loop_start().as_secs_f64();

        //Event handling
        for event in self.event_pump.poll_iter() 
        {
            match event
            {
                Event::Quit { .. } => return false,
                _ => {}
            }
        }

        self.prev_keys = self.keys.clone();
        self.keys = self.event_pump.keyboard_state().pressed_scancodes().filter_map(Keycode::from_scancode).collect();

        return true;
    }

    pub fn is_key_down(&self, key: Keycode) -> bool
    {
        self.keys.contains(&key) && !self.prev_keys.contains(&key)
    }

    pub fn is_key_pressed(&self, key: Keycode) -> bool
    {
        self.keys.contains(&key)
    }

    pub fn is_key_up(&self, key: Keycode) -> bool
    {
        !self.keys.contains(&key) && self.prev_keys.contains(&key)
    }

    pub fn loop_end(&mut self)
    {
        self.loop_helper.loop_sleep();
    }
}

pub struct Sprite<'a>
{
    texture: Texture<'a>,
    
    pub dst: Rect,
    pub src: Rect,
    pub pivot: Point,
    pub angle: f64,
    pub alpha: u8,
    
    pub hflip: bool,
    pub vflip: bool
}

#[allow(dead_code)]
impl<'a> Sprite<'a>
{
    pub fn from_file(texture_creator: &'a TextureCreator<WindowContext>, path: &'a str) -> Sprite<'a>
    {
        let texture = texture_creator.load_texture(path).expect("Failed to load texture!");

        let query = texture.query();
        let w = query.width;
        let h = query.height;

        Sprite 
        { 
            texture, 
            dst: Rect::new(0, 0, w, h), 
            src: Rect::new(0, 0, w, h), 
            pivot: Point::new(0, 0), 
            angle: 0.0, 
            alpha: 255,
            hflip: false, 
            vflip: false
        }
    }

    pub fn from_texture(texture: Texture<'a>) -> Sprite<'a>
    {
        let query = texture.query();
        let w = query.width;
        let h = query.height;

        Sprite 
        { 
            texture, 
            dst: Rect::new(0, 0, w, h), 
            src: Rect::new(0, 0, w, h), 
            pivot: Point::new(0, 0), 
            angle: 0.0, 
            alpha: 255,
            hflip: false, 
            vflip: false
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>)
    {
        self.texture.set_alpha_mod(self.alpha);
        
        canvas.copy_ex(&self.texture, self.src, self.dst, self.angle, self.pivot, self.hflip, self.vflip).expect("Failed to copy texture!");
    }

    pub fn draw_offset(&mut self, canvas: &mut Canvas<Window>, offset: Point)
    {
        self.texture.set_alpha_mod(self.alpha);

        let mut new_rect = self.dst.clone();
        new_rect.set_x(new_rect.x() + offset.x());
        new_rect.set_y(new_rect.y() + offset.y());
        canvas.copy_ex(&self.texture, self.src, new_rect, self.angle, self.pivot, self.hflip, self.vflip).expect("Failed to copy texture!");
    }

    pub fn set_blend_mod(&mut self, mode: BlendMode)
    {
        self.texture.set_blend_mode(mode);
    }

    pub fn set_color_mod(&mut self, mode: Color)
    {
        self.texture.set_color_mod(mode.r, mode.g, mode.b);
    }
}

pub struct PointF
{
    pub x: f64,
    pub y: f64
}