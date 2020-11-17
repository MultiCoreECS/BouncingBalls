use std::sync::Arc;
use rand::prelude::*;
use clap::{Arg, App};

use SmolECS::{
    entity::*,
    system::*,
    rayon::*,
    world::*,
};


// STRUCTS

pub struct WorldBounds{
    x: i32, 
    y: i32
}

pub struct Time{
    beginning: std::time::Instant,
    last: std::time::Instant,
    total: f64,
    delta: f64
}

pub struct SimType {
    collisions: bool
}


// COMPONENTS

#[derive(Copy, Clone)]
pub struct Velocity{
    x: f32,
    y: f32,
}

#[derive(Copy, Clone)]
pub struct Position{
    x: f32, 
    y: f32
}

#[derive(Copy, Clone)]
pub struct Color{
    r: i32, 
    b: i32,
    g: i32
}

#[derive(Copy, Clone)]
pub struct Radius(f32);


// SYSTEMS

pub struct UpdateTime;
impl<'d, 'w: 'd> System<'d, 'w, World> for UpdateTime{
    type SystemData = 
        Write<'d, Time>
    ;

    fn run(&self, mut time: Self::SystemData) {
        let current = std::time::Instant::now();
        time.delta = current.duration_since(time.last).as_secs_f64();
        time.total = current.duration_since(time.beginning).as_secs_f64();
        time.last = current;
        println!("{}", time.delta as f32);
    }
}

pub struct ApplyVelocities;
impl<'d, 'w: 'd> System<'d, 'w, World> for ApplyVelocities{
    type SystemData = (
        WriteComp<'d, Velocity>,
        Read<'d, Time>,
        WriteComp<'d, Position>
    );

    fn run(&self, (mut vels, time, mut positions): Self::SystemData) {
        for (vel, position) in (&mut vels, &mut positions).join(){
            position.x += vel.x * time.delta as f32;
            position.y += vel.y * time.delta as f32;
        }
    }
}

// update to work on surfaces also or create a diff function for surfaces?
fn ball_collision_check(rad_one: &Radius, pos_one: &Position, rad_two: &Radius, pos_two: &Position) -> bool{
    (pos_two.x - pos_one.x).powi(2) + (pos_two.y - pos_one.y).powi(2) <= (rad_one.0 + rad_two.0).powi(2)
}

use std::ops::Deref;
pub struct BallCollisionCheck;
impl<'d, 'w: 'd> System<'d, 'w, World> for BallCollisionCheck{
    type SystemData = (
        ReadComp<'d, Radius>,
        ReadComp<'d, Position>,
        WriteComp<'d, Velocity>,
        Read<'d, WorldBounds>,
        Read<'d, SimType>,
        WriteComp<'d, Color>,
        Read<'d, EntityStorage>
    );

    fn run(&self, (radii, positions, mut vels, bounds, sim_type, mut colors, ents): Self::SystemData) {
        let mut rng = rand::thread_rng();
        let mut add_colors = Vec::new();
        let mut remove_colors = Vec::new();

        for (pos_one, rad_one, vel_one, ent_one) in (&positions, &radii, &mut vels, ents.deref()).join(){
            for(pos_two, rad_two, ent_two) in (&positions, &radii, ents.deref()).join(){
                if ball_collision_check(rad_one, pos_one, rad_two, pos_two) && ent_one != ent_two{
                    vel_one.x *= -1.0;
                    vel_one.y *= -1.0;

                    if sim_type.collisions {
                        for (_color, ent) in (&mut colors, ents.deref()).join() {
                            if ent == ent_one {
                                remove_colors.push(ent_one);
                            }
                            if ent == ent_two {
                                remove_colors.push(ent_two);
                            }
                        }
                        for ent in (ents.deref()).join() {
                            if ent == ent_one {
                                add_colors.push(ent_one);
                            }
                            if ent == ent_two {
                                add_colors.push(ent_two);
                            }
                        }
                    }
                    break;
                }
            }
            // do wall collision check here
            if pos_one.x < 0.0 || pos_one.x > bounds.x as f32 {
                vel_one.x *= -1.0;
            }

            if pos_one.y < 0.0 || pos_one.y > bounds.y as f32 {
                vel_one.y *= -1.0;
            }
        }

        for ent in remove_colors.drain(..) {
            ent.remove(&mut colors);
        }
        for ent in add_colors.drain(..) {
            ent.add(&mut colors, Color{r: rng.gen_range(0, 255), b: rng.gen_range(0, 255), g: rng.gen_range(0, 255)});
        }
    }
}


// make things happen

fn main() {

    let app = App::new("balls")
        .version("1.0")
        .about("simulates balls bouncing in a room")
        .author("SmolECS")
        .arg(Arg::with_name("size")
            .short("s")
            .long("size")
            .help("the length of one side of the square room to be simulated")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("type")
            .short("t")
            .long("type")
            .help("type '1' for regular bouncing balls simulation; type '2' for component updating upon collision bouncing balls simulation")
            .takes_value(true)
            .required(true))
        .get_matches();
    
    
    let size = app.value_of("size").unwrap_or("100");
    let sim_type = app.value_of("type").unwrap_or("1");

    let s: i32 = size.parse().unwrap();
    let t: i32 = sim_type.parse().unwrap();

    let mut world = World::new();
    world.register_comp::<Velocity>();
    world.register_comp::<Position>();
    world.register_comp::<Radius>();
    world.register_comp::<Color>();

    world.insert(WorldBounds{x: s, y: s});
    world.insert(Time{
        beginning: std::time::Instant::now(),
        last: std::time::Instant::now(),
        total: 0.0,
        delta: 0.0,
    });
    world.insert(EntityStorage::new());

    if t == 2 {
        world.insert(SimType{collisions: true});
    }
    else {
        world.insert(SimType{collisions: false});
    }

    let mut ents = Write::<EntityStorage>::get_data(&world);
    let mut positions = WriteComp::<Position>::get_data(&world);
    let mut vels = WriteComp::<Velocity>::get_data(&world);
    let mut radius = WriteComp::<Radius>::get_data(&world);

    let mut rng = rand::thread_rng();
    for _i in 0..s*s {
        ents.create_entity()
            .add(&mut positions, Position{x: rng.gen_range(0.0, s as f32), y: rng.gen_range(0.0, s as f32)})
            .add(&mut vels, Velocity{x: rng.gen_range(-1.0, 1.0), y: rng.gen_range(-1.0, 1.0)})
            .add(&mut radius, Radius(0.5));
    }

    let mut scheduler = SystemScheduler::new(Arc::new(ThreadPoolBuilder::new().num_threads(4).build().unwrap()));
    scheduler.add(UpdateTime{}, "update_time", vec![]);
    scheduler.add(ApplyVelocities{}, "update_positions", vec!["update_time"]);
    scheduler.add(BallCollisionCheck{}, "collision_check", vec!["update_positions"]);

    drop(ents);
    drop(positions);
    drop(vels);
    drop(radius);

    for _i in 0..10000 {
        scheduler.run(&world);

    }

    let finish = Read::<Time>::get_data(&world);
}
