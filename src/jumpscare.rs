use sdl2::{mixer::Channel, render::{TextureCreator}, video::WindowContext};
use crate::engine::{Sprite, Video, Audio, EventLoop};

pub fn jumpscare(video: &mut Video, audio: &Audio, event: &mut EventLoop, texture_creator: &TextureCreator<WindowContext>, state: &mut i32)
{
    let jumpscare_snd = audio.load_sound("assets/sounds/jumpscare.wav");
    Channel::all().play(&jumpscare_snd, 0).unwrap();

    let mut jumpscare_texture = Sprite::from_file(&texture_creator, "assets/sprites/jumpscare.png");
    jumpscare_texture.dst.set_width(64);
    jumpscare_texture.src.set_width(64);

    let mut cnt = 0.0;
    loop 
    {
        if !event.loop_start() { *state = -1; break; }
        video.canvas_mut().clear();

        jumpscare_texture.src.set_x((cnt as i32).clamp(0, 4) * 64);
        jumpscare_texture.draw(video.canvas_mut());

        video.canvas_mut().present();
        
        cnt += 0.2;
        if cnt > 30.0
        {
            *state = 1;
            break;
        }

        event.loop_end();
    }
}