#![windows_subsystem = "windows"]
use engine::{Init, Video, EventLoop, Audio};
use game::game;
use jumpscare::jumpscare;

mod engine;
mod game;
mod jumpscare;

pub fn main() 
{
    //Engine stuff
    let init = Init::new();
    let mut video = Video::new(init.context(), 64, 64);
    let mut event = EventLoop::new(init.context(), 60);
    let audio = Audio::new();
    let texture_creator = video.canvas().texture_creator();
    video.canvas_mut().window_mut().set_size(640, 640).unwrap();
    video.canvas_mut().window_mut().maximize();
    
    let mut state = 1;
    'state_loop: loop {
        match state {
            -1 => break 'state_loop,
            1 => game(&mut video, &audio, &mut event, &texture_creator, &mut state),
            2 => jumpscare(&mut video, &audio, &mut event, &texture_creator, &mut state),
            _ => {}
        }
    }
}
