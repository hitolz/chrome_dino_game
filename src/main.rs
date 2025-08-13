use bevy::prelude::*;
use rand::prelude::*;

// 游戏常量
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 400.0;
const GROUND_Y: f32 = -150.0;
const GRAVITY: f32 = -1200.0;
const JUMP_SPEED: f32 = 500.0;
const GAME_SPEED: f32 = 300.0;

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "小恐龙跳跃游戏".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9))) // 灰白色背景
        .init_state::<GameState>()
        .insert_resource(ObstacleTimer(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (setup_camera, load_assets))
        .add_systems(PostStartup, (spawn_ground, spawn_player))
        .add_systems(
            Update,
            (
                player_input,
                apply_gravity,
                move_obstacles,
                spawn_obstacles,
                check_collisions,
                update_score,
                despawn_offscreen,
                spawn_ground_tiles,
                animate_dino,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, restart_game.run_if(in_state(GameState::GameOver)))
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
}

// 游戏逻辑系统
fn player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Player, &Transform)>,
) {
    if let Ok((mut player, transform)) = player_query.single_mut() {
        // 检查是否在地面上（用于判断是否可以跳跃）
        let on_ground = transform.translation.y <= GROUND_Y + 30.0;

        if (keyboard_input.just_pressed(KeyCode::Space)
            || keyboard_input.just_pressed(KeyCode::ArrowUp))
            && on_ground
        {
            player.velocity_y = JUMP_SPEED;
            player.is_jumping = true;
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
        // 随机生成障碍物
        let mut rng = rand::rng();
        if rng.gen_bool(0.85) {
            // 85% 概率生成障碍物
            // 随机选择仙人掌类型
            let cactus_index = rng.gen_range(0..assets.cactus_textures.len());
            let selected_cactus = assets.cactus_textures[cactus_index].clone();

            // 随机生成不同大小的仙人掌，增加游戏多样性
            let (width, height) = match cactus_index {
                0 => (25.0, 45.0), // cactus1 - 较小
                1 => (35.0, 55.0), // cactus2 - 较大
                _ => (30.0, 50.0), // 默认大小
            };

            commands.spawn((
                Sprite {
                    image: selected_cactus,
                    custom_size: Some(Vec2::new(width, height)),
                    ..default()
                },
                Transform::from_xyz(500.0, GROUND_Y + 0.0 + height / 2.0, 1.0),
                Obstacle { scored: false },
                Velocity {
                    x: -GAME_SPEED,
                    y: 0.0,
                },
            ));
        }

        // 设置下一个障碍物的随机间隔时间
        // 间隔范围：0.8秒到2.5秒，确保最小间隔足够恐龙跳过
        let next_interval = rng.gen_range(0.5..1.8);
        timer
            .0
            .set_duration(std::time::Duration::from_secs_f32(next_interval));
        timer.0.reset();
    }
}

fn check_collisions(
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Transform, (With<Player>, Without<Obstacle>)>,
    obstacle_query: Query<&Transform, (With<Obstacle>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.single() {
        for obstacle_transform in obstacle_query.iter() {
            let distance = player_transform
                .translation
                .distance(obstacle_transform.translation);

            // 简单的圆形碰撞检测
            if distance < 30.0 {
                next_state.set(GameState::GameOver);
                break;
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
    for (entity, transform) in query.iter() {
        if transform.translation.x < -500.0 {
            commands.entity(entity).despawn();
        }
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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    entities: Query<Entity, Or<(With<Obstacle>, With<GameScore>, With<Ground>, With<Player>)>>,
    assets: Res<GameAssets>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
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
