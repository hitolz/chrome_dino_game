use bevy::prelude::*;
use rand::prelude::*;

// 游戏常量
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 400.0;
const GROUND_Y: f32 = -150.0;
const GRAVITY: f32 = -1200.0;
const JUMP_SPEED: f32 = 500.0;
const GAME_SPEED: f32 = 300.0;
const TARGET_FPS: f64 = 60.0; // 目标帧率，适合大多数显示器

// 游戏状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

// 组件定义
#[derive(Component)]
struct Player {
    velocity_y: f32,
    is_jumping: bool,
    jump_cooldown: f32, // 跳跃冷却时间
}

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct DinoAnimation {
    frames: Vec<Handle<Image>>,
    current_frame: usize,
}

#[derive(Component)]
struct Obstacle {
    scored: bool, // 是否已经计分
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Velocity {
    x: f32,
    #[allow(dead_code)]
    y: f32,
}

#[derive(Component)]
struct GameScore {
    value: u32,
}

// 资源定义
#[derive(Resource)]
struct GameAssets {
    dino_frames: Vec<Handle<Image>>,
    cactus_textures: Vec<Handle<Image>>,
    ground_texture: Handle<Image>,
}

#[derive(Resource)]
struct ObstacleTimer(Timer);

// 输入状态资源
#[derive(Resource)]
struct InputState {
    space_pressed: bool,
    space_just_pressed: bool,
    #[allow(dead_code)]
    last_jump_time: f32,
}

// 性能监控资源
#[derive(Resource)]
struct PerformanceStats {
    frame_time: f32,
    fps: f32,
    frame_count: u32,
    last_fps_update: f32,
    min_fps: f32,
    max_fps: f32,
    frame_time_samples: Vec<f32>, // 用于计算帧时间稳定性
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct GameOverText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "小恐龙跳跃游戏".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                present_mode: bevy::window::PresentMode::Fifo, // 固定垂直同步，避免高刷新率问题
                ..default()
            }),
            ..default()
        }).set(bevy::render::RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(
                bevy::render::settings::WgpuSettings {
                    features: bevy::render::settings::WgpuFeatures::default(),
                    limits: bevy::render::settings::WgpuLimits::downlevel_webgl2_defaults(),
                    // 优化：启用更好的多重采样抗锯齿
                    ..default()
                }
            ),
            ..default()
        }))
        // 添加帧率插件来更好地控制渲染
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / TARGET_FPS)
        ))
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9))) // 灰白色背景
        .init_state::<GameState>()
        .insert_resource(ObstacleTimer(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .insert_resource(InputState {
            space_pressed: false,
            space_just_pressed: false,
            last_jump_time: 0.0,
        })
        .insert_resource(PerformanceStats {
            frame_time: 0.0,
            fps: 60.0,
            frame_count: 0,
            last_fps_update: 0.0,
            min_fps: 60.0,
            max_fps: 60.0,
            frame_time_samples: Vec::with_capacity(60),
        })
        .add_systems(Startup, (setup_camera, load_assets))
        .add_systems(PostStartup, (spawn_ground, spawn_player))
        .add_systems(
            Update,
            (
                update_performance_stats,
                handle_input,
                player_input,
                apply_gravity,
                move_obstacles,
                spawn_obstacles,
                check_collisions,
                update_score,
                despawn_offscreen,
                spawn_ground_tiles,
                animate_dino,
                update_fps_display,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, (handle_input, restart_game, show_game_over_screen).run_if(in_state(GameState::GameOver)))
        .run();
}

// 初始化系统
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载恐龙动画帧
    let dino_frames = vec![
        asset_server.load("sprites/dino1.png"),
        asset_server.load("sprites/dino2.png"),
    ];

    // 加载多种仙人掌图片
    let cactus_textures = vec![
        asset_server.load("sprites/cactus1.png"),
        asset_server.load("sprites/cactus2.png"),
    ];

    let ground_texture = asset_server.load("sprites/ground.png");

    commands.insert_resource(GameAssets {
        dino_frames,
        cactus_textures,
        ground_texture,
    });
}

fn spawn_ground(mut commands: Commands, assets: Res<GameAssets>) {
    // 计算需要覆盖的范围：从屏幕左边延伸到右边，再多加一些缓冲
    let start_x = -WINDOW_WIDTH / 2.0 - 200.0; // 屏幕左边缘再往左200px
    let end_x = WINDOW_WIDTH / 2.0 + 400.0; // 屏幕右边缘再往右400px
    let tile_width = 100.0;
    let tile_count = ((end_x - start_x) / tile_width).ceil() as i32;

    // 生成连续的地面块，确保完全覆盖
    for i in 0..tile_count {
        commands.spawn((
            Sprite {
                image: assets.ground_texture.clone(),
                color: Color::srgb(0.55, 0.27, 0.07), // 棕色色调
                custom_size: Some(Vec2::new(tile_width, 20.0)),
                ..default()
            },
            Transform::from_xyz(start_x + i as f32 * tile_width, GROUND_Y, 0.0),
            Ground,
            Velocity {
                x: -GAME_SPEED,
                y: 0.0,
            },
        ));
    }
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Sprite {
            image: assets.dino_frames[0].clone(),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(-300.0, GROUND_Y + 30.0, 1.0),
        Player {
            velocity_y: 0.0,
            is_jumping: false,
            jump_cooldown: 0.0,
        },
        DinoAnimation {
            frames: assets.dino_frames.clone(),
            current_frame: 0,
        },
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
    ));

    // 生成分数显示
    commands.spawn((
        GameScore { value: 0 },
        Text2d::new("Score: 0"),
        Transform::from_xyz(-350.0, 150.0, 1.0),
    ));

    // 生成FPS显示
    commands.spawn((
        FpsText,
        Text2d::new("FPS: 60"),
        Transform::from_xyz(300.0, 150.0, 1.0),
        TextFont {
            font_size: 20.0,
            ..default()
        },
    ));
}

// 优化的输入处理系统
fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    _time: Res<Time>,
    mut input_state: ResMut<InputState>,
) {
    
    // 检测空格键状态
    let space_pressed_now = keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::ArrowUp);
    
    // 更新输入状态
    input_state.space_just_pressed = space_pressed_now && !input_state.space_pressed;
    input_state.space_pressed = space_pressed_now;
}

// 游戏逻辑系统
fn player_input(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut player_query: Query<(&mut Player, &Transform)>,
) {
    if let Ok((mut player, transform)) = player_query.single_mut() {
        let _current_time = time.elapsed_secs();
        
        // 更新跳跃冷却时间
        if player.jump_cooldown > 0.0 {
            player.jump_cooldown -= time.delta_secs();
        }
        
        // 检查是否在地面上（用于判断是否可以跳跃）
        let on_ground = transform.translation.y <= GROUND_Y + 30.0;

        // 使用优化的输入检测
        if input_state.space_just_pressed
            && on_ground 
            && player.jump_cooldown <= 0.0
        {
            player.velocity_y = JUMP_SPEED;
            player.is_jumping = true;
            player.jump_cooldown = 0.1; // 设置跳跃冷却时间
        }
    }
}

fn apply_gravity(time: Res<Time>, mut player_query: Query<(&mut Player, &mut Transform)>) {
    if let Ok((mut player, mut transform)) = player_query.single_mut() {
        // 应用重力
        player.velocity_y += GRAVITY * time.delta_secs();

        // 更新位置
        transform.translation.y += player.velocity_y * time.delta_secs();

        // 检查是否着地
        if transform.translation.y <= GROUND_Y + 30.0 {
            transform.translation.y = GROUND_Y + 30.0;
            player.velocity_y = 0.0;
            player.is_jumping = false;
        }
    }
}

fn move_obstacles(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity), Without<Player>>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_secs();
    }
}

fn spawn_obstacles(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ObstacleTimer>,
    assets: Res<GameAssets>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        // 使用thread_local的RNG，避免重复初始化
        thread_local! {
            static RNG: std::cell::RefCell<rand::rngs::ThreadRng> = std::cell::RefCell::new(rand::rng());
        }
        
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            
            if rng.random_bool(0.85) {
                // 85% 概率生成障碍物
                // 预计算的障碍物配置，避免运行时计算
                static CACTUS_CONFIGS: [(f32, f32); 2] = [
                    (25.0, 45.0), // cactus1 - 较小
                    (35.0, 55.0), // cactus2 - 较大
                ];
                
                let cactus_index = rng.random_range(0..assets.cactus_textures.len());
                let (width, height) = CACTUS_CONFIGS[cactus_index];

                commands.spawn((
                    Sprite {
                        image: assets.cactus_textures[cactus_index].clone(),
                        custom_size: Some(Vec2::new(width, height)),
                        ..default()
                    },
                    Transform::from_xyz(500.0, GROUND_Y + height * 0.5, 1.0),
                    Obstacle { scored: false },
                    Velocity {
                        x: -GAME_SPEED,
                        y: 0.0,
                    },
                ));
            }

            // 设置下一个障碍物的随机间隔时间
            let next_interval = rng.random_range(0.5..1.8);
            timer.0.set_duration(std::time::Duration::from_secs_f32(next_interval));
            timer.0.reset();
        });
    }
}

fn check_collisions(
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Transform, (With<Player>, Without<Obstacle>)>,
    obstacle_query: Query<&Transform, (With<Obstacle>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation;
        
        // 优化：只检查玩家附近的障碍物
        for obstacle_transform in obstacle_query.iter() {
            let obstacle_pos = obstacle_transform.translation;
            
            // 早期退出：如果障碍物太远，跳过
            let dx = (player_pos.x - obstacle_pos.x).abs();
            if dx > 50.0 {
                continue;
            }
            
            let dy = (player_pos.y - obstacle_pos.y).abs();
            if dy > 50.0 {
                continue;
            }
            
            // 更精确的矩形碰撞检测
            let collision_threshold = 25.0;
            if dx < collision_threshold && dy < collision_threshold {
                next_state.set(GameState::GameOver);
                return; // 早期退出
            }
        }
    }
}

fn update_score(
    mut score_query: Query<(&mut GameScore, &mut Text2d)>,
    mut obstacle_query: Query<(&mut Obstacle, &Transform), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<Obstacle>)>,
) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok((mut score, mut text)) = score_query.single_mut() {
            // 检测是否有障碍物被跳过
            for (mut obstacle, obstacle_transform) in obstacle_query.iter_mut() {
                if !obstacle.scored
                    && obstacle_transform.translation.x < player_transform.translation.x
                {
                    obstacle.scored = true;
                    score.value += 1; // 跳过一个障碍物得1分
                    text.0 = format!("Score: {}", score.value);
                }
            }
        }
    }
}

fn despawn_offscreen(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<Velocity>, Without<Player>)>,
) {
    // 批量收集需要删除的实体，减少commands调用
    let mut entities_to_despawn = Vec::with_capacity(8);
    
    for (entity, transform) in query.iter() {
        if transform.translation.x < -500.0 {
            entities_to_despawn.push(entity);
        }
    }
    
    // 批量删除实体
    for entity in entities_to_despawn {
        commands.entity(entity).despawn();
    }
}

fn spawn_ground_tiles(
    mut commands: Commands,
    assets: Res<GameAssets>,
    ground_query: Query<&Transform, With<Ground>>,
) {
    // 找到最右边的地面块
    let mut rightmost_x = -400.0;
    for transform in ground_query.iter() {
        if transform.translation.x > rightmost_x {
            rightmost_x = transform.translation.x;
        }
    }

    // 如果最右边的地面块位置小于窗口右边缘，就生成新的地面块
    if rightmost_x < WINDOW_WIDTH / 2.0 + 100.0 {
        for i in 0..3 {
            commands.spawn((
                Sprite {
                    image: assets.ground_texture.clone(),
                    color: Color::srgb(0.55, 0.27, 0.07),
                    custom_size: Some(Vec2::new(100.0, 20.0)),
                    ..default()
                },
                Transform::from_xyz(rightmost_x + 100.0 + (i as f32 * 100.0), GROUND_Y, 0.0),
                Ground,
                Velocity {
                    x: -GAME_SPEED,
                    y: 0.0,
                },
            ));
        }
    }
}

fn restart_game(
    mut input_state: ResMut<InputState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut obstacle_timer: ResMut<ObstacleTimer>,
    entities: Query<Entity, Or<(With<Obstacle>, With<GameScore>, With<Ground>, With<Player>, With<FpsText>, With<GameOverText>)>>,
    assets: Res<GameAssets>,
) {
    if input_state.space_just_pressed {
        // 清除所有游戏实体
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }

        // 重新生成地面 - 使用与初始生成相同的逻辑
        let start_x = -WINDOW_WIDTH / 2.0 - 200.0; // 屏幕左边缘再往左200px
        let end_x = WINDOW_WIDTH / 2.0 + 400.0; // 屏幕右边缘再往右400px
        let tile_width = 100.0;
        let tile_count = ((end_x - start_x) / tile_width).ceil() as i32;

        for i in 0..tile_count {
            commands.spawn((
                Sprite {
                    image: assets.ground_texture.clone(),
                    color: Color::srgb(0.55, 0.27, 0.07),
                    custom_size: Some(Vec2::new(tile_width, 20.0)),
                    ..default()
                },
                Transform::from_xyz(start_x + i as f32 * tile_width, GROUND_Y, 0.0),
                Ground,
                Velocity {
                    x: -GAME_SPEED,
                    y: 0.0,
                },
            ));
        }

        // 重新生成恐龙
        commands.spawn((
            Sprite {
                image: assets.dino_frames[0].clone(),
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            Transform::from_xyz(-300.0, GROUND_Y + 30.0, 1.0),
            Player {
                velocity_y: 0.0,
                is_jumping: false,
                jump_cooldown: 0.0,
            },
            DinoAnimation {
                frames: assets.dino_frames.clone(),
                current_frame: 0,
            },
            AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        ));

        // 重新生成分数显示
        commands.spawn((
            GameScore { value: 0 },
            Text2d::new("Score: 0"),
            Transform::from_xyz(-350.0, 150.0, 1.0),
        ));

        // 重新生成FPS显示
        commands.spawn((
            FpsText,
            Text2d::new("FPS: 60"),
            Transform::from_xyz(300.0, 150.0, 1.0),
            TextFont {
                font_size: 20.0,
                ..default()
            },
        ));

        // 重置输入状态，避免立即再次重启
        input_state.space_just_pressed = false;
        input_state.space_pressed = false;

        // 重置障碍物计时器
        obstacle_timer.0.set_duration(std::time::Duration::from_secs_f32(2.0));
        obstacle_timer.0.reset();

        next_state.set(GameState::Playing);
    }
}

fn animate_dino(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut DinoAnimation,
        &mut Sprite,
        &Player,
    )>,
) {
    for (mut timer, mut animation, mut sprite, player) in query.iter_mut() {
        // 只有在地面上才播放跑步动画
        if !player.is_jumping {
            timer.0.tick(time.delta());

            if timer.0.just_finished() {
                // 切换到下一帧
                animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
                sprite.image = animation.frames[animation.current_frame].clone();
            }
        } else {
            // 跳跃时固定在第一帧
            animation.current_frame = 0;
            sprite.image = animation.frames[0].clone();
        }
    }
}

// 性能监控系统
fn update_performance_stats(
    time: Res<Time>,
    mut perf_stats: ResMut<PerformanceStats>,
) {
    let frame_time = time.delta_secs();
    perf_stats.frame_time = frame_time;
    perf_stats.frame_count += 1;
    
    // 收集帧时间样本用于稳定性分析
    perf_stats.frame_time_samples.push(frame_time);
    if perf_stats.frame_time_samples.len() > 60 {
        perf_stats.frame_time_samples.remove(0);
    }
    
    let current_time = time.elapsed_secs();
    if current_time - perf_stats.last_fps_update >= 1.0 {
        let elapsed = current_time - perf_stats.last_fps_update;
        perf_stats.fps = perf_stats.frame_count as f32 / elapsed;
        
        // 更新最小/最大FPS
        perf_stats.min_fps = perf_stats.min_fps.min(perf_stats.fps);
        perf_stats.max_fps = perf_stats.max_fps.max(perf_stats.fps);
        
        perf_stats.frame_count = 0;
        perf_stats.last_fps_update = current_time;
    }
}

// FPS显示更新系统
fn update_fps_display(
    perf_stats: Res<PerformanceStats>,
    mut fps_query: Query<&mut Text2d, With<FpsText>>,
) {
    if let Ok(mut text) = fps_query.single_mut() {
        text.0 = format!("FPS: {:.0}", perf_stats.fps);
    }
}

// 显示游戏结束屏幕
fn show_game_over_screen(
    mut commands: Commands,
    game_over_query: Query<Entity, With<GameOverText>>,
) {
    // 如果还没有游戏结束文本，就创建一个
    if game_over_query.is_empty() {
        commands.spawn((
            GameOverText,
            Text2d::new("Game Over! Press SPACE to restart"),
            Transform::from_xyz(0.0, 0.0, 10.0),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.0, 0.0)), // 红色
        ));
    }
}
