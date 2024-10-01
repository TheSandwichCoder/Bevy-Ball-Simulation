#[cfg(not(target_arch = "wasm32"))]
use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy::window::PrimaryWindow;
use std::collections::HashMap;
use rand::Rng;

use crate::settings::*;

pub struct BallPlugin;

impl Plugin for BallPlugin{
    fn build(&self, app: &mut App){
        app.add_plugins((
            #[cfg(not(target_arch = "wasm32"))]
            Wireframe2dPlugin,
        ))
            .add_systems(Startup, spawn_ball_parent)

            .add_systems(Update, (
                spawn_ball,
                update_gravity_velocity,
                update_processes,
                interact,
            ))
            .add_systems(Update, update_ball_draw_position)
            .register_type::<Ball>();
    }
}

#[derive(Component)]
pub struct BallParent;

#[derive(Component, Default, Reflect, Clone, Copy)]
pub struct Ball{
    pub size : f32,
    pub pos : Vec3,
    pub velocity : Vec3,
    pub elasticity: f32,
    pub id: i32,
}

fn spawn_ball_parent(mut commands: Commands){
    commands.spawn((SpatialBundle::default(), BallParent, Name::new("Ball Parent")));
}

fn spawn_ball(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    parent: Query<Entity, With<BallParent>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    if !input.pressed(KeyCode::Space){
        return;
    }

    let parent = parent.single();
    
    let shape = Mesh2dHandle(meshes.add(Circle{radius: 10.0}));
    let color = Color::rgb(rand::thread_rng().gen_range(0.0..1.0), rand::thread_rng().gen_range(0.0..1.0), rand::thread_rng().gen_range(0.0..1.0));

    // Spawns the ball 
    commands.entity(parent).with_children(|commands|{
        commands.spawn((
            MaterialMesh2dBundle{
                mesh: shape,
                material: materials.add(color),
                transform: Transform{
                    translation: Vec3::new(0.0, 180.0, 0.0),
                    ..default()
                },
                ..default()
            },
            Ball{
                size: 10.0,
                pos: Vec3::new(0.0, 180.0, 0.0),
                velocity: Vec3::new(10.0,10.0,0.0),
                elasticity: 0.3,
                id: rand::thread_rng().gen_range(1..=100000000),
            },
            Name::new("Ball"),
        ));
    });

    info!("Spawned new Ball");
}

fn interact(
    mut ballObjectQuery: Query<&mut Ball>,
    time: Res<Time>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mouseInput: Res<ButtonInput<MouseButton>>,
    keyInput: Res<ButtonInput<KeyCode>>,
){

    if mouseInput.pressed(MouseButton::Right) {
        let windowPosition = q_windows.single().cursor_position();

        let position: Vec3 = Vec3::new(windowPosition.unwrap().x, windowPosition.unwrap().y, 0.0);
        
        let mut rel_position: Vec3 = position - HALF_DIM;
        rel_position.y = -rel_position.y;

        for mut ballObject in &mut ballObjectQuery{
            let relPos = (rel_position - ballObject.pos).normalize();
            // info!("{}",relPos);
            ballObject.velocity += relPos * 400.0 * time.delta_seconds();
        }

        let mousePosI: IVec3 = position.as_ivec3() / CHUNK_SIZE;

        info!("NEW CHECK");
        for x in  -1..2{
            for y in -1..2{
                let offsetVec: IVec3 = IVec3::new(x, y, 0);
                let check_chunk_pos: IVec3 = mousePosI + offsetVec;
                
                if !in_bounds(&check_chunk_pos){
                    continue;
                }
                // info!("{} {}", check_chunk_pos, vec2d_to_index(&check_chunk_pos))
            }
        }

        
    }
}


fn update_processes(
    mut ballObjectQuery: Query<&mut Ball>,
    time: Res<Time>,
){
    
    let mut i = 0;

    while i < ITERATION_COUNT {
        update_ball_position(&mut ballObjectQuery, &time, ITERATION_DELTA);
        container_collision(&mut ballObjectQuery);
        // ball_collision_physics(&mut ballObjectQuery);
        ball_collision_physics_optimised(&mut ballObjectQuery);
        i += 1;
    }    
}

fn update_ball_position(
    mut ballObjectQuery: &mut Query<&mut Ball>,
    time: &Res<Time>,
    iteration_delta: f32,
){
    for mut ballObject in ballObjectQuery{
        let ballVelocity = ballObject.velocity;
        ballObject.pos += ballVelocity * time.delta_seconds() * iteration_delta;
    }   
}

fn update_gravity_velocity(
    mut ballObjectQuery: Query<&mut Ball>,
    time: Res<Time>,
){
    for mut ballObject in &mut ballObjectQuery{
        ballObject.velocity -= GRAVITY * time.delta_seconds();
    }
}

fn container_collision(
    mut ballObjectQuery: &mut Query<&mut Ball>,
){
    for mut ballObject in ballObjectQuery{
        if ballObject.pos.y - ballObject.size < -HALF_DIM.y + 100.0{
            ballObject.pos.y = -HALF_DIM.y + ballObject.size + 100.0;
            ballObject.velocity.y = -ballObject.velocity.y * ballObject.elasticity;
        }

        else if ballObject.pos.y + ballObject.size > HALF_DIM.y{
            ballObject.pos.y = HALF_DIM.y - ballObject.size;
            ballObject.velocity.y = -ballObject.velocity.y * ballObject.elasticity;
        }

        if ballObject.pos.x + ballObject.size > HALF_DIM.x{
            ballObject.pos.x = HALF_DIM.x - ballObject.size;
            ballObject.velocity.x = -ballObject.velocity.x * ballObject.elasticity;
        }
        else if ballObject.pos.x - ballObject.size < -HALF_DIM.x + 100.0{
            ballObject.pos.x = -HALF_DIM.x + ballObject.size + 100.0;
            ballObject.velocity.x = -ballObject.velocity.x * ballObject.elasticity;
        }
    }
}

fn is_ball_collision(
    ball1: &Ball,
    ball2: &Ball,
) -> bool{
    let distanceVec:Vec3 = ball1.pos - ball2.pos;
    let radiusSum: f32 = ball1.size + ball2.size;
    
    return distanceVec.length_squared() <= radiusSum*radiusSum;
}

fn add_to_hashmap<'a>(
    hashmap: &mut HashMap<i32, Vec<&'a mut Ball>>,
    hash: i32,
    item: &'a mut Ball,
){
    hashmap.entry(hash).or_insert_with(Vec::new).push(item);
}

fn vec2d_to_index(
    vector: &IVec3,
) -> i32{
    let mut x: i32 = vector.x;
    let mut y: i32 = vector.y;

    if vector.x < 0{
        x = 0;
    }
    else if vector.x >= BALL_CHUNK_ARRAY_DIM.x{
        x = BALL_CHUNK_ARRAY_DIM.x - 1;
    }
    
    if vector.y < 0{
        y = 0;
    }
    else if vector.y >= BALL_CHUNK_ARRAY_DIM.y{
        y = BALL_CHUNK_ARRAY_DIM.y - 1;
    }

    return BALL_CHUNK_ARRAY_DIM.x * y + x;
}

const BALL_CHUNK_ARRAY_DIM: IVec3 = IVec3::new(65, 37, 0);
const BALL_CHUNK_ARRAY_LENGTH: usize = (BALL_CHUNK_ARRAY_DIM.x * BALL_CHUNK_ARRAY_DIM.y + BALL_CHUNK_ARRAY_DIM.x) as usize;

fn in_bounds(
    vector: &IVec3,
) -> bool{
    return vector.x < BALL_CHUNK_ARRAY_DIM.x && vector.x >= 0 && vector.y < BALL_CHUNK_ARRAY_DIM.y && vector.y >= 0;
}

fn ball_collision_physics_optimised(
    mut ballObjectQuery: &mut Query<&mut Ball>,
    // mut query_set: ParamSet<(Query<&mut Ball>, Query<&Ball>)>
){
    let mut ball_chunk_array: [Vec<Ball>; BALL_CHUNK_ARRAY_LENGTH] = std::array::from_fn(|_| Vec::new());

    for ballObject in ballObjectQuery.iter(){
        let screen_pos: IVec3 = (ballObject.pos + HALF_DIM).as_ivec3();
        let chunk_pos: IVec3 = screen_pos / CHUNK_SIZE;
        let chunk_index: usize = vec2d_to_index(&chunk_pos) as usize; 
        
        ball_chunk_array[chunk_index].push(ballObject.clone());
    }
    // info!("START COLLISION");
    for mut ballObject1 in ballObjectQuery.iter_mut(){
        // info!("new ball col");
        let screen_pos: IVec3 = (ballObject1.pos + HALF_DIM).as_ivec3();
        let chunk_pos: IVec3 = screen_pos / CHUNK_SIZE;
        let chunk_index: usize = vec2d_to_index(&chunk_pos) as usize;
        
        for x in  -1..2{
            for y in -1..2{
                let offsetVec: IVec3 = IVec3::new(x, y, 0);
                let check_chunk_pos: IVec3 = chunk_pos + offsetVec;
                
                if !in_bounds(&check_chunk_pos){
                    continue;
                }

                let chunk_index: usize = vec2d_to_index(&check_chunk_pos) as usize;
                
                for ballObject2 in &ball_chunk_array[chunk_index]{
                    // same object
                    if ballObject1.id == ballObject2.id{
                        continue;
                    }

                    if is_ball_collision(&ballObject1, &ballObject2){
                        
                        let ball_rel_vec:Vec3 = ballObject1.pos - ballObject2.pos;
                        
                        // if ball_rel_vec.length() == 0.0{
                        //     // continue;
                        //     ballObject1.pos += Vec3::new(rand::thread_rng().gen_range(0.0..1.0), 0.0, 0.0);
                        //     continue;
                        // }
                        
                        // info!("COLLISION HERE");
                        
                        let rel_distance: f32 = ballObject1.size + ballObject2.size - ball_rel_vec.length();
                        let ball_rel_vec_normalised: Vec3 = ball_rel_vec.normalize();
                        let average_elastivity: f32 = (ballObject1.elasticity + ballObject2.elasticity) / 2.0;
                        
                        let d1: Vec3 = ball_rel_vec_normalised * (rel_distance/2.0);
                        let d2: Vec3 = ball_rel_vec_normalised * rel_distance * average_elastivity * 10.0;
        
                        ballObject1.pos += d1;
                        // ballObject2.pos -= d1;
        
                        ballObject1.velocity += d2;
                        // ballObject2.velocity -= d2;

                        if ballObject1.pos.x.is_nan(){
                            
                            info!("PROBLEM: {} {} {}", chunk_index, chunk_pos, ballObject1.pos);
                        }
                    }
                }

            }
        } 
    }
}

fn ball_collision_physics(
    mut ballObjectQuery: &mut Query<&mut Ball>
){

    let mut iter = ballObjectQuery.iter_combinations_mut();
    let mut have_collisions = true;

    while have_collisions{
        have_collisions = false;

        while let Some([mut ballObject1, mut ballObject2]) =
            iter.fetch_next()
        {
            if is_ball_collision(&ballObject1, &ballObject2){
                have_collisions = true;
                let ball_rel_vec:Vec3 = ballObject1.pos - ballObject2.pos;
                
                let rel_distance: f32 = ballObject1.size + ballObject1.size - ball_rel_vec.length();
                let ball_rel_vec_normalised: Vec3 = ball_rel_vec.normalize();
                let average_elastivity: f32 = (ballObject1.elasticity + ballObject2.elasticity) / 2.0;
                
                let d1: Vec3 = ball_rel_vec_normalised * (rel_distance/2.0);
                let d2: Vec3 = ball_rel_vec_normalised * rel_distance * average_elastivity * 10.0;

                ballObject1.pos += d1;
                ballObject2.pos -= d1;

                ballObject1.velocity += d2;
                ballObject2.velocity -= d2;
            }
        }
    }
}


fn update_ball_draw_position(
    mut ballTransformQuery: Query<(&mut Transform, &Ball)>,
){
    let mut counter: i32 = 0;
    for (mut transform, ballObject) in &mut ballTransformQuery{
        //info!("{}: {}", counter, ballObject.pos);
        transform.translation = ballObject.pos;
        counter += 1;
    }
}
