use rand::{rngs::ThreadRng, Rng};
use sdl2::{rect::{Rect, Point}, mixer::Channel, keyboard::Keycode, render::{BlendMode, TextureCreator}, pixels::Color, video::WindowContext};

use crate::engine::{PointF, Sprite, Video, Audio, EventLoop};

pub struct Particle
{
    pub pos: PointF,
    pub lifetime: u8,
    pub color: Color,
    pub light: bool
}

pub struct Candle
{
    pub dst: Rect,
    pub lit: bool
}

pub struct Ghost<'a>
{
    pub pos: PointF,
    pub spr: Sprite<'a>,
    pub spd: f64
}

pub fn game(video: &mut Video, audio: &Audio, event: &mut EventLoop, texture_creator: &TextureCreator<WindowContext>, state: &mut i32)
{
    let mut rand = rand::thread_rng();
    let ch0 = Channel(0);
    ch0.set_volume(32);

    let ch1 = Channel(1);
    let ch2 = Channel(2);

    //Map & Player section
    let mut map = [(); 4096].map(|_| 0);
    let mut candles = Vec::<Candle>::new();
    let mut player = Sprite::from_file(&texture_creator, "assets/sprites/player.png");
    let mut player_pos = PointF { x: 0.0, y: 0.0 }; 
    let mut player_anim: f64 = 0.0;
    let mut particles = Vec::<Particle>::new(); 

    //Enemy
    let mut ghost = Ghost {spd: 1.0, pos: PointF { x: 0.0, y: 0.0 }, spr: Sprite::from_file(&texture_creator, "assets/sprites/ghost.png") };
    ghost.spr.alpha = 0;

    gen_map(&mut player_pos, &mut map, &mut candles, &mut ghost, &mut rand);

    player.src.set_width(16);
    player.src.set_height(10);
    player.dst.set_width(16);
    player.dst.set_height(10);

    //Lights
    let mut light_texture = texture_creator.create_texture_target(None, 64, 64).unwrap();
    light_texture.set_blend_mode(BlendMode::Mod);

    //Load assets
    let fireamb = audio.load_music("assets/sounds/fireambient.wav");
    let ambient = audio.load_sound("assets/sounds/ambient2.ogg");
    let matches = audio.load_sound("assets/sounds/match.wav");
    let ghost_snd = audio.load_sound("assets/sounds/ghost.ogg");

    fireamb.play(-1).unwrap();
    ch0.play(&ambient, -1).unwrap();

    let mut tilemap = Sprite::from_file(&texture_creator, "assets/sprites/objects.png");
    let mut light = Sprite::from_file(&texture_creator, "assets/sprites/light.png");
    let mut pixel = Sprite::from_file(&texture_creator, "assets/sprites/pixel.png");
    light.set_blend_mod(BlendMode::Add);    

    let mut timer: f64 = 0.0;
    let mut match_timer: f64 = 0.0;
    let mut map_toggle = false;
    loop 
    {
        if !event.loop_start() { *state = -1; break; }
        video.canvas_mut().clear();
        
        let mut c_x = 0;
        let mut c_y = 0;
        update_player(&event, video, &mut player, &mut player_anim, &mut player_pos, &mut c_x, &mut c_y);

        //Spawn particles
        if timer as i32 % 2 == 0
        {
            let mut offset = 4.0;
    
            if player.hflip 
            { 
                offset = 11.0; 
            }
        
            let mut color = Color::GRAY;
            let mut light = false;
            if timer as i32 % rand.gen_range(20..30) == 0
            {
                color = Color::YELLOW;
                light = true;
            }

            spawn_particles(&mut particles, &mut rand, light, player_pos.x + offset, player_pos.y + 2.0, color);
        }
                

        //Setup tilemap for drawing
        tilemap.src.set_width(16);
        tilemap.src.set_height(16);
        tilemap.dst.set_width(16);
        tilemap.dst.set_height(16);
        
        //Draw map
        let c_xx = (c_x / 16) as usize;
        let c_yy = (c_y / 16) as usize;
        for i in 0..5
        {
            for j in 0..5
            {
                let x = i + c_xx;
                let y = j + c_yy;
                let v = map[x + y * 64];
                if v == 0 { continue; }
                
                tilemap.dst.set_x((x * 16) as i32 - c_x);
                tilemap.dst.set_y((y * 16) as i32 - c_y);
                tilemap.src.set_x((v as i32 - 1) * 16);
                tilemap.draw(video.canvas_mut());                       
            }
        }

        let player_rect = Rect::new(player_pos.x as i32 + 5, player_pos.y as i32 + 5, 6, 5);

        //Update and Draw candles
        for i in 0..candles.len()
        {
            let candle = candles.get_mut(i).unwrap();

            //Setup tilemap
            tilemap.src = Rect::new(32, 0, 6, 4);
            tilemap.dst = Rect::new(candle.dst.x() - c_x, candle.dst.y() - c_y, 6, 4);
            tilemap.draw(video.canvas_mut());

            if candle.dst.has_intersection(player_rect) && !candle.lit && event.is_key_pressed(Keycode::Z)
            {
                match_timer += 0.01;

                //Draw progress bar
                pixel.dst.set_x(candle.dst.x() - c_x);
                pixel.dst.set_y(candle.dst.y() - c_y - 8);
                pixel.dst.set_width(candle.dst.width() - 2);
                pixel.dst.set_height(1);
                pixel.set_color_mod(Color::RED);
                pixel.draw(video.canvas_mut());


                pixel.dst.set_width((match_timer * (candle.dst.width() - 2) as f64) as u32);
                pixel.set_color_mod(Color::GREEN);
                pixel.draw(video.canvas_mut());

                if match_timer > 1.0
                {
                    if ghost.spd < 4.0
                    {
                        ghost.spd += 0.5;
                    }

                    match_timer = 0.0; 
                    candle.lit = true;
                    ch2.play(&matches, 0).unwrap();
                }
            } 
            else if !event.is_key_pressed(Keycode::Z)
            {
                match_timer = 0.0;  
            }

            if candle.lit && timer as i32 % 4 == 0
            {
                let mut color = Color::GRAY;
                let mut light = false;
                
                if timer as i32 % rand.gen_range(20..30) == 0
                {
                    color = Color::YELLOW;
                    light = true;
                }

                spawn_particles(&mut particles, &mut rand, light, candle.dst.x() as f64 + 2.0, candle.dst.y() as f64, color);
            }
        }

        if player_rect.has_intersection(Rect::new(ghost.pos.x as i32, ghost.pos.y as i32, 8, 8))
        {
            sdl2::mixer::Music::halt();
            *state = 2;
            break;
        }
        
        //Particles
        for i in 0..particles.len()
        {
            let part = match particles.get_mut(i)
            {
                Some(part) => part,
                None => continue,
            };

            part.lifetime += 1;
            part.pos.y -= 0.7;
            
            if part.lifetime >= 255
            {
                particles.remove(i);
                continue;
            }

            if part.pos.x <= (c_x).into() || part.pos.y <= (c_y).into() || part.pos.x > (c_x + 64).into() || part.pos.y > (c_y + 64).into()
            {
                continue;
            }
            
            pixel.dst.set_width(1);
            pixel.dst.set_height(1);
            pixel.set_color_mod(part.color);
            pixel.dst.set_x(part.pos.x as i32 - c_x);
            pixel.dst.set_y(part.pos.y as i32 - c_y);
            pixel.draw(video.canvas_mut());
        }

        //Draw light
        video.canvas_mut().with_texture_canvas(&mut light_texture, |canvas|
        {
            canvas.clear();

            //Player light
            let size = (45.0 + timer.sin() * 3.0) as i32;
            light.dst.set_x(player_pos.x as i32 + 8 - c_x - (size / 2));
            light.dst.set_y(player_pos.y as i32 + 5 - c_y - (size / 2));
            light.dst.set_width(size as u32);
            light.dst.set_height(size as u32);
            light.draw(canvas);

            //Particle light
            for i in 0..particles.len()
            {
                let part = particles.get_mut(i).unwrap();
                if !part.light { continue; }

                if part.pos.x <= (c_x).into() || part.pos.y <= (c_y).into() || part.pos.x > (c_x + 64).into() || part.pos.y > (c_y + 64).into()
                {
                    continue;
                }

                let size = 6;
                light.dst.set_x(part.pos.x as i32 - c_x - (size / 2));
                light.dst.set_y(part.pos.y as i32 - c_y - (size / 2));
                light.dst.set_width(size as u32);
                light.dst.set_height(size as u32);
                light.draw(canvas);
            }

            //Candle light
            let size = (16.0 + timer.sin() * 3.0) as i32;
            light.dst.set_width(size as u32);
            light.dst.set_height(size as u32);

            for i in 0..candles.len()
            {
                let candle = candles.get_mut(i).unwrap();
                if !candle.lit { continue; }

                light.dst.set_x(candle.dst.x() + 3 - c_x - (size / 2));
                light.dst.set_y(candle.dst.y() + 2 - c_y - (size / 2));
                light.draw(canvas);
            }

        }).unwrap();

        //Update and draw ghost
        {
            update_ghost(&mut ghost, &player_pos);
            
            if ghost.pos.x as i32 % 8 == 0 || ghost.pos.y as i32 % 8 == 0
            {
                spawn_particles(&mut particles, &mut rand, true, ghost.pos.x + 2.0, ghost.pos.y + 1.0, Color::RED);
            }

            if (ghost.pos.x as i32 % 8 == 0 || ghost.pos.y as i32 % 8 == 0) && ghost.spr.alpha == 0
            {
                ch1.play(&ghost_snd, 0).unwrap();
                ghost.spr.alpha = 255;
            }

            //"3D" sound
            {
                let xx = ghost.pos.x - player_pos.x;
                let a_x = xx.clamp(-32.0, 32.0) / 32.0;
                let yy = ghost.pos.y - player_pos.y;
                
                let left = (a_x * 255.0 - 255.0).abs().clamp(0.0, 255.0) as u8;
                let right = (a_x * 255.0) as u8;
                ch1.set_panning(left, right).unwrap();

                let vol = 128 - (xx + yy).abs().clamp(0.0, 128.0) as i32;
                ch1.set_volume(vol);
            }

            ghost.spr.dst.set_x(ghost.pos.x as i32 / 8 * 8);
            ghost.spr.dst.set_y(ghost.pos.y as i32 / 8 * 8);
            ghost.spr.draw_offset(video.canvas_mut(), Point::new(-c_x, -c_y));
        }

        //Draw player
        {
            let mut offset = 4;
            if player.hflip { offset = -4; }
            player.src.set_x((player_anim as i32) * 16);
            player.dst.set_x(player_pos.x as i32 + offset);
            player.dst.set_y(player_pos.y as i32);
            player.draw_offset(video.canvas_mut(), Point::new(-c_x, -c_y));
        }

        timer += 0.5;

        //Map
        if event.is_key_down(Keycode::A)
        { map_toggle = !map_toggle; }

        //Draw map
        if map_toggle
        {
            video.canvas_mut().clear();

            for i in 0..64
            {
                for j in 0..64
                {
                    let tile = map[i + j * 64];
                    let color = match tile
                    {
                        1 => Color::RGB(0, 94, 41),
                        2 => Color::RGB(94, 25, 0),
                        _ => Color::BLACK
                    };

                    pixel.dst.set_width(1);
                    pixel.dst.set_height(1);
                    pixel.dst.set_x(i as i32);
                    pixel.dst.set_y(j as i32);

                    pixel.set_color_mod(color);
                    pixel.draw(video.canvas_mut());
                }
            }

            for i in 0..candles.len()
            {
                let candle = candles.get(i).unwrap();
                if candle.lit { continue; }

                pixel.dst.set_x(candle.dst.x() / 16);
                pixel.dst.set_y(candle.dst.y() / 16);

                pixel.set_color_mod(Color::RGB(219, 227, 0));
                pixel.draw(video.canvas_mut());
            }

            pixel.dst.set_x((player_pos.x as i32 + 5) / 16);
            pixel.dst.set_y((player_pos.y as i32 + 5) / 16);

            pixel.set_color_mod(Color::RGB(230, 11, 0));
            pixel.draw(video.canvas_mut());

            video.canvas_mut().present();
            event.loop_end();
            continue;
        }

        //Draw light texture
        video.canvas_mut().copy(&light_texture, None, None).unwrap();
        
        video.canvas_mut().present();
        event.loop_end();
    }
}

pub fn gen_map(player: &mut PointF, map: &mut [u8; 4096], candles: &mut Vec<Candle>, ghost: &mut Ghost,  rand: &mut ThreadRng)
{
    for i in 0..64
    {
        for j in 0..64
        {
            let r = rand.gen_range(0..2);
            map[i + j * 64] = r;
        }
    }

    player.x = (rand.gen_range(0..64) * 16) as f64;
    player.y = (rand.gen_range(0..64) * 16) as f64;

    ghost.pos.x = player.x + (rand.gen_range(0..360) as f64).sin() * (rand.gen_range(4..8) * 16) as f64;
    ghost.pos.y = player.y + (rand.gen_range(0..360) as f64).sin() * (rand.gen_range(4..8) * 16) as f64;

    for _i in 0..(rand.gen_range(8..12) as usize)
    {
        'search: loop 
        {
            let r_x = rand.gen_range(0..64);
            let r_y = rand.gen_range(0..64);
            let tile = map[r_x + r_y * 64];

            if tile == 0
            {
                let candle = Candle { dst: Rect::new(r_x as i32 * 16, r_y as i32 * 16, 8, 8), lit: false };
                candles.push(candle);
                break 'search;
            } 
        }
    }
}

pub fn update_player(event: &EventLoop, video: &mut Video, player: &mut Sprite, player_anim: &mut f64, player_pos: &mut PointF, c_x: &mut i32, c_y: &mut i32)
{
    //Control
    let mut walking = false;

    if event.is_key_pressed(Keycode::Left)
    {
        player_pos.x -= 0.5;
        player.hflip = false;
        walking = true;
    }
    else if event.is_key_pressed(Keycode::Right)
    {
        player_pos.x += 0.5;
        player.hflip = true;
        walking = true;
    }

    if event.is_key_pressed(Keycode::Up)
    {
        player_pos.y -= 0.5;
        walking = true;
    }
    else if event.is_key_pressed(Keycode::Down)
    {
        player_pos.y += 0.5 ;
        walking = true;
    }

    //Animate player
    if walking
    {
        if *player_anim < 4.0 {
            *player_anim = 4.0;
        }

        *player_anim += 0.2;

        if *player_anim > 7.0 {
            *player_anim = 4.0;
        }

    } 
    else 
    {
        if *player_anim >= 4.0 {
            *player_anim = 0.0;
        }

        *player_anim += 0.2;

        if *player_anim > 3.0 {
            *player_anim = 0.0;
        }
    }

    //Clamp player's position
    player_pos.x = player_pos.x.clamp(-5.0, 64.0 * 16.0 - 25.0);
    player_pos.y = player_pos.y.clamp(-5.0, 64.0 * 16.0 - 25.0);

    //Set camera to new position
    *c_x = (player_pos.x - 32.0 + 8.0).clamp(0.0, 944.0) as i32;
    *c_y = (player_pos.y - 32.0 + 5.0).clamp(0.0, 944.0) as i32;
}

fn update_ghost(ghost: &mut Ghost, player: &PointF)
{
    let dir_x = ((player.x + 8.0) - ghost.pos.x).signum(); 
    let dir_y = ((player.y + 5.0) - ghost.pos.y).signum(); 

    ghost.pos.x += dir_x * 0.1 * ghost.spd;
    ghost.pos.y += dir_y * 0.1 * ghost.spd;
    
    if ghost.spr.alpha > 0
    {
        ghost.spr.alpha -= 1;
    }
}

fn spawn_particles(particles: &mut Vec<Particle>, rand: &mut ThreadRng, emit_light: bool, x: f64, y: f64, color: Color)
{    
    let part = Particle 
    { 
        light: emit_light, /* Emit lights */ 
        pos: PointF { x: x + rand.gen_range(-1..2) as f64, y: y + 2.0 }, 
        lifetime: 200, 
        color  
    };

    particles.push(part);
}