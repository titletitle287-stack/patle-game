use macroquad::prelude::*;

const BASE_H: f32 = 540.0;
const CASTLE_W: f32 = 130.0;
const CASTLE_H: f32 = 160.0;
const GROUND: f32 = 420.0;
const UI_TOP: f32 = 466.0;
const MAX_PARTS: usize = 650;
const TAU: f32 = 6.2831855;
const NW: usize = 5;
const NH: usize = 3;
const NU: usize = 10;

// =====================================================================
//  helpers
// =====================================================================

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(r, g, b, a)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn with_a(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

fn shade(c: Color, f: f32) -> Color {
    Color::new((c.r * f).min(1.0), (c.g * f).min(1.0), (c.b * f).min(1.0), c.a)
}

fn rnd(a: f32, b: f32) -> f32 {
    rand::gen_range(a, b)
}

fn fmt(v: f64) -> String {
    let sfx = ["", "K", "M", "B", "T", "Q"];
    let mut x = v;
    let mut i = 0;
    while x >= 1000.0 && i < 5 {
        x /= 1000.0;
        i += 1;
    }
    if i == 0 {
        format!("{:.0}", x)
    } else {
        format!("{:.1}{}", x, sfx[i])
    }
}

#[derive(Clone, Copy)]
struct G {
    u: f32,
    ox: f32,
    oy: f32,
}

impl G {
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        draw_rectangle((x + self.ox) * self.u, (y + self.oy) * self.u, w * self.u, h * self.u, c);
    }
    fn frame(&self, x: f32, y: f32, w: f32, h: f32, th: f32, c: Color) {
        draw_rectangle_lines((x + self.ox) * self.u, (y + self.oy) * self.u, w * self.u, h * self.u, th * self.u, c);
    }
    fn circ(&self, x: f32, y: f32, r: f32, c: Color) {
        draw_circle((x + self.ox) * self.u, (y + self.oy) * self.u, r * self.u, c);
    }
    fn ring(&self, x: f32, y: f32, r: f32, th: f32, c: Color) {
        draw_circle_lines((x + self.ox) * self.u, (y + self.oy) * self.u, r * self.u, th * self.u, c);
    }
    fn glow(&self, x: f32, y: f32, r: f32, c: Color) {
        for i in 0..4 {
            let k = 1.0 - i as f32 * 0.25;
            self.circ(x, y, r * k, with_a(c, c.a * 0.22));
        }
    }
    fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, th: f32, c: Color) {
        draw_line(
            (x1 + self.ox) * self.u,
            (y1 + self.oy) * self.u,
            (x2 + self.ox) * self.u,
            (y2 + self.oy) * self.u,
            th * self.u,
            c,
        );
    }
    fn tri(&self, a: (f32, f32), b: (f32, f32), c: (f32, f32), color: Color) {
        let u = self.u;
        draw_triangle(
            vec2((a.0 + self.ox) * u, (a.1 + self.oy) * u),
            vec2((b.0 + self.ox) * u, (b.1 + self.oy) * u),
            vec2((c.0 + self.ox) * u, (c.1 + self.oy) * u),
            color,
        );
    }
    fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Color) {
        draw_text(s, (x + self.ox) * self.u, (y + self.oy) * self.u, size * self.u, c);
    }
    fn text_w(&self, s: &str, size: f32) -> f32 {
        measure_text(s, None, (size * self.u) as u16, 1.0).width / self.u
    }
    fn text_center(&self, s: &str, cx: f32, y: f32, size: f32, c: Color) {
        let w = self.text_w(s, size);
        self.text(s, cx - w / 2.0, y, size, c);
    }
    fn text_right(&self, s: &str, rx: f32, y: f32, size: f32, c: Color) {
        let w = self.text_w(s, size);
        self.text(s, rx - w, y, size, c);
    }
}

// =====================================================================
//  biomes
// =====================================================================

struct Biome {
    name: &'static str,
    top: Color,
    bot: Color,
    far: Color,
    near: Color,
    grass: Color,
    tuft: Color,
    dirt: Color,
    amb: u8,
    moon: Color,
}

fn biome(w: u32) -> Biome {
    match ((w.max(1) - 1) / 10) % 5 {
        0 => Biome {
            name: "Twilight Plains",
            top: rgba(0.03, 0.04, 0.16, 1.0),
            bot: rgba(0.55, 0.25, 0.42, 1.0),
            far: rgba(0.17, 0.12, 0.30, 1.0),
            near: rgba(0.10, 0.14, 0.25, 1.0),
            grass: rgba(0.14, 0.32, 0.15, 1.0),
            tuft: rgba(0.22, 0.48, 0.22, 1.0),
            dirt: rgba(0.22, 0.14, 0.10, 1.0),
            amb: 0,
            moon: rgba(0.97, 0.96, 0.85, 1.0),
        },
        1 => Biome {
            name: "Scorched Desert",
            top: rgba(0.12, 0.08, 0.28, 1.0),
            bot: rgba(0.98, 0.55, 0.22, 1.0),
            far: rgba(0.45, 0.22, 0.22, 1.0),
            near: rgba(0.55, 0.33, 0.20, 1.0),
            grass: rgba(0.78, 0.62, 0.32, 1.0),
            tuft: rgba(0.88, 0.72, 0.40, 1.0),
            dirt: rgba(0.50, 0.35, 0.18, 1.0),
            amb: 1,
            moon: rgba(1.0, 0.85, 0.40, 1.0),
        },
        2 => Biome {
            name: "Frozen Peaks",
            top: rgba(0.04, 0.08, 0.22, 1.0),
            bot: rgba(0.55, 0.75, 0.90, 1.0),
            far: rgba(0.55, 0.65, 0.80, 1.0),
            near: rgba(0.40, 0.52, 0.70, 1.0),
            grass: rgba(0.85, 0.92, 1.0, 1.0),
            tuft: rgba(1.0, 1.0, 1.0, 1.0),
            dirt: rgba(0.35, 0.42, 0.55, 1.0),
            amb: 2,
            moon: rgba(0.90, 0.95, 1.0, 1.0),
        },
        3 => Biome {
            name: "Volcanic Wastes",
            top: rgba(0.08, 0.01, 0.02, 1.0),
            bot: rgba(0.85, 0.25, 0.05, 1.0),
            far: rgba(0.25, 0.06, 0.06, 1.0),
            near: rgba(0.18, 0.07, 0.07, 1.0),
            grass: rgba(0.22, 0.12, 0.10, 1.0),
            tuft: rgba(0.45, 0.15, 0.08, 1.0),
            dirt: rgba(0.14, 0.07, 0.06, 1.0),
            amb: 3,
            moon: rgba(1.0, 0.45, 0.20, 1.0),
        },
        _ => Biome {
            name: "The Void",
            top: rgba(0.01, 0.0, 0.06, 1.0),
            bot: rgba(0.35, 0.10, 0.55, 1.0),
            far: rgba(0.14, 0.06, 0.25, 1.0),
            near: rgba(0.09, 0.04, 0.18, 1.0),
            grass: rgba(0.18, 0.10, 0.30, 1.0),
            tuft: rgba(0.40, 0.20, 0.65, 1.0),
            dirt: rgba(0.10, 0.05, 0.18, 1.0),
            amb: 4,
            moon: rgba(0.75, 0.50, 1.0, 1.0),
        },
    }
}

// =====================================================================
//  weapons, heroes, upgrades
// =====================================================================

#[derive(Clone, Copy)]
struct Weapon {
    name: &'static str,
    desc: &'static str,
    price: u64,
    cd: f32,
    dmg: f32,
    speed: f32,
    grav: f32,
    pierce: u32,
    splash: f32,
    magic: bool,
    kind: u8,
}

fn weapon(i: usize) -> Weapon {
    match i {
        0 => Weapon { name: "Hunter Bow", desc: "Fast arrows that pierce enemies", price: 0, cd: 0.26, dmg: 22.0, speed: 1000.0, grav: 220.0, pierce: 2, splash: 0.0, magic: false, kind: 0 },
        1 => Weapon { name: "Heavy Crossbow", desc: "Huge bolts, pierce many enemies", price: 900, cd: 0.50, dmg: 85.0, speed: 1500.0, grav: 40.0, pierce: 5, splash: 0.0, magic: false, kind: 1 },
        2 => Weapon { name: "Fire Staff", desc: "Fireballs with area damage", price: 3000, cd: 0.42, dmg: 70.0, speed: 700.0, grav: 0.0, pierce: 1, splash: 80.0, magic: true, kind: 2 },
        3 => Weapon { name: "Storm Staff", desc: "Chain lightning, ignores armor", price: 9000, cd: 0.32, dmg: 55.0, speed: 0.0, grav: 0.0, pierce: 0, splash: 0.0, magic: true, kind: 3 },
        _ => Weapon { name: "Siege Cannon", desc: "Giant blasts, massive damage", price: 25000, cd: 1.0, dmg: 420.0, speed: 800.0, grav: 300.0, pierce: 1, splash: 130.0, magic: false, kind: 4 },
    }
}

fn hero_info(i: usize) -> (&'static str, &'static str, u64) {
    match i {
        0 => ("Ranger", "+10% crit chance", 0),
        1 => ("Knight", "+60% castle HP, +10% damage", 2500),
        _ => ("Archmage", "+25% gold, skills 20% faster", 6000),
    }
}

const UP_NAME: [&str; NU] = [
    "Damage", "Fire Rate", "Multishot", "Crit Chance", "Crit Power", "Pierce", "Castle HP", "HP Regen", "Gold Bonus", "Skill Power",
];
const UP_DESC: [&str; NU] = [
    "+20% damage per level",
    "+10% shooting speed",
    "+1 projectile (max 8)",
    "+4% crit chance (max 18)",
    "+15% crit damage",
    "+1 pierce (max 8)",
    "+25% castle HP and heal",
    "+1 HP per second (max 40)",
    "+10% gold from kills",
    "Skills +10% dmg, -4% cooldown",
];
const UP_BASE: [f64; NU] = [30.0, 40.0, 150.0, 50.0, 60.0, 100.0, 35.0, 60.0, 80.0, 90.0];
const UP_GROW: [f64; NU] = [1.32, 1.34, 2.0, 1.4, 1.4, 1.9, 1.3, 1.4, 1.5, 1.5];
const UP_MAX: [u32; NU] = [u32::MAX, u32::MAX, 8, 18, u32::MAX, 8, u32::MAX, 40, u32::MAX, 20];

fn up_cost(i: usize, lvl: u32) -> u64 {
    (UP_BASE[i] * UP_GROW[i].powf(lvl as f64)) as u64
}

// =====================================================================
//  enemies
// =====================================================================

#[derive(Clone, Copy)]
struct Kind {
    hp: f32,
    speed: f32,
    dmg: f32,
    reward: f32,
    w: f32,
    h: f32,
    color: Color,
    flying: bool,
    range: f32,
    armor: bool,
    splits: bool,
    bomb: bool,
    heal: bool,
    summon: bool,
    boss: bool,
}

fn base(hp: f32, speed: f32, dmg: f32, reward: f32, w: f32, h: f32, color: Color) -> Kind {
    Kind {
        hp, speed, dmg, reward, w, h, color,
        flying: false, range: 0.0, armor: false, splits: false, bomb: false, heal: false, summon: false, boss: false,
    }
}

fn kind(k: usize) -> Kind {
    match k {
        0 => base(20.0, 48.0, 5.0, 3.0, 24.0, 32.0, rgba(0.35, 0.72, 0.30, 1.0)),
        1 => base(14.0, 100.0, 3.0, 4.0, 20.0, 26.0, rgba(0.95, 0.45, 0.20, 1.0)),
        2 => base(95.0, 32.0, 12.0, 9.0, 38.0, 48.0, rgba(0.25, 0.50, 0.25, 1.0)),
        3 => Kind { flying: true, ..base(12.0, 78.0, 3.0, 5.0, 28.0, 18.0, rgba(0.60, 0.40, 0.85, 1.0)) },
        4 => Kind { range: 250.0, ..base(28.0, 42.0, 6.0, 8.0, 24.0, 40.0, rgba(0.88, 0.88, 0.80, 1.0)) },
        5 => Kind { splits: true, ..base(50.0, 30.0, 6.0, 6.0, 30.0, 26.0, rgba(0.30, 0.85, 0.50, 1.0)) },
        6 => base(32.0, 150.0, 6.0, 7.0, 38.0, 24.0, rgba(0.55, 0.55, 0.60, 1.0)),
        7 => Kind { armor: true, ..base(150.0, 30.0, 14.0, 16.0, 32.0, 46.0, rgba(0.60, 0.65, 0.75, 1.0)) },
        8 => Kind { bomb: true, ..base(26.0, 72.0, 60.0, 10.0, 26.0, 30.0, rgba(0.85, 0.20, 0.20, 1.0)) },
        9 => Kind { flying: true, ..base(65.0, 58.0, 10.0, 14.0, 32.0, 40.0, rgba(0.40, 0.60, 0.95, 1.0)) },
        10 => Kind { heal: true, range: 170.0, ..base(55.0, 34.0, 4.0, 14.0, 28.0, 42.0, rgba(0.20, 0.75, 0.70, 1.0)) },
        11 => base(14.0, 36.0, 3.0, 2.0, 18.0, 16.0, rgba(0.30, 0.85, 0.50, 1.0)),
        12 => Kind { boss: true, ..base(3000.0, 20.0, 45.0, 400.0, 96.0, 124.0, rgba(0.50, 0.32, 0.18, 1.0)) },
        13 => Kind { boss: true, flying: true, range: 330.0, ..base(2600.0, 26.0, 40.0, 450.0, 120.0, 70.0, rgba(0.75, 0.15, 0.15, 1.0)) },
        _ => Kind { boss: true, range: 340.0, summon: true, ..base(2300.0, 22.0, 35.0, 450.0, 60.0, 112.0, rgba(0.45, 0.20, 0.60, 1.0)) },
    }
}

// (kind, first wave, weight)
const SPAWN: [(usize, u32, u32); 11] = [
    (0, 1, 40),
    (1, 2, 22),
    (3, 3, 14),
    (2, 4, 14),
    (6, 5, 14),
    (4, 6, 12),
    (5, 7, 12),
    (7, 8, 10),
    (8, 9, 10),
    (9, 11, 8),
    (10, 12, 8),
];

struct Enemy {
    id: u32,
    kind: usize,
    x: f32,
    hp: f32,
    max_hp: f32,
    hit: f32,
    lane: f32,
    yo: f32,
    fy: f32,
    ph: f32,
    atk: f32,
    spec: f32,
    slow: f32,
    elite: bool,
    norew: bool,
    dm: f32,
}

fn enemy_rect(e: &Enemy, t: f32) -> Rect {
    let kd = kind(e.kind);
    if kd.flying {
        let cy = e.fy + (t * 3.0 + e.ph).sin() * 8.0;
        Rect::new(e.x - kd.w / 2.0, cy - kd.h / 2.0, kd.w, kd.h)
    } else {
        Rect::new(e.x - kd.w / 2.0, GROUND + e.yo - kd.h, kd.w, kd.h)
    }
}

// =====================================================================
//  effects and projectiles
// =====================================================================

struct Particle {
    p: Vec2,
    v: Vec2,
    life: f32,
    max: f32,
    c: Color,
    size: f32,
    grav: f32,
}

struct FText {
    p: Vec2,
    val: f32,
    crit: bool,
    life: f32,
}

struct Bolt {
    pts: Vec<Vec2>,
    life: f32,
    c: Color,
}

struct Ring {
    p: Vec2,
    r: f32,
    life: f32,
    max: f32,
    c: Color,
}

struct Meteor {
    p: Vec2,
    v: Vec2,
    alive: bool,
}

struct Proj {
    p: Vec2,
    v: Vec2,
    dmg: f32,
    crit: bool,
    kind: u8,
    pierce: u32,
    splash: f32,
    grav: f32,
    magic: bool,
    alive: bool,
    hit: Vec<u32>,
    life: f32,
}

struct EProj {
    p: Vec2,
    v: Vec2,
    dmg: f32,
    kind: u8,
    alive: bool,
}

fn burst(parts: &mut Vec<Particle>, p: Vec2, n: usize, c: Color, speed: f32, life: f32) {
    for _ in 0..n {
        if parts.len() >= MAX_PARTS {
            return;
        }
        let a = rnd(0.0, TAU);
        let s = rnd(0.25, 1.0) * speed;
        parts.push(Particle {
            p,
            v: vec2(a.cos() * s, a.sin() * s - speed * 0.25),
            life,
            max: life,
            c,
            size: rnd(2.0, 4.5),
            grav: 380.0,
        });
    }
}

fn jagged(pts: &[Vec2]) -> Vec<Vec2> {
    let mut out: Vec<Vec2> = Vec::new();
    if pts.is_empty() {
        return out;
    }
    out.push(pts[0]);
    for w in 0..pts.len() - 1 {
        let a = pts[w];
        let b = pts[w + 1];
        let d = b - a;
        let len = d.length().max(1.0);
        let n = ((len / 22.0).ceil() as i32).max(1);
        let perp = vec2(-d.y / len, d.x / len);
        for k in 1..n {
            let f = k as f32 / n as f32;
            let off = rnd(-9.0, 9.0);
            out.push(a + d * f + perp * off);
        }
        out.push(b);
    }
    out
}

fn hero_origin() -> Vec2 {
    vec2(82.0, GROUND - CASTLE_H - 24.0)
}

fn solve_aim(o: Vec2, tgt: Vec2, tv: Vec2, speed: f32, grav: f32) -> Vec2 {
    let mut p = tgt;
    let mut t = (tgt - o).length() / speed;
    for _ in 0..3 {
        p = tgt + tv * t;
        t = (p - o).length() / speed;
    }
    let drop = 0.5 * grav * t * t;
    let mut d = vec2(p.x, p.y - drop) - o;
    if d.x < 40.0 {
        d.x = 40.0;
    }
    d.normalize()
}

// =====================================================================
//  ui layout
// =====================================================================

fn btn_shop() -> Rect {
    Rect::new(10.0, UI_TOP + 8.0, 130.0, 56.0)
}
fn btn_auto() -> Rect {
    Rect::new(150.0, UI_TOP + 8.0, 120.0, 56.0)
}
fn btn_skill(lw: f32, i: usize) -> Rect {
    Rect::new(lw - 10.0 - (3 - i) as f32 * 130.0 + 8.0, UI_TOP + 8.0, 122.0, 56.0)
}
fn shop_panel(lw: f32) -> Rect {
    let pw = (lw - 24.0).min(980.0);
    Rect::new((lw - pw) / 2.0, 14.0, pw, 512.0)
}
fn tab_rect(p: Rect, i: usize) -> Rect {
    Rect::new(p.x + 10.0 + i as f32 * 160.0, p.y + 10.0, 152.0, 44.0)
}
fn close_rect(p: Rect) -> Rect {
    Rect::new(p.x + p.w - 120.0, p.y + 10.0, 110.0, 44.0)
}
// (items, cols, rows, max height)
fn tab_grid(tab: usize) -> (usize, usize, usize, f32) {
    match tab {
        0 => (NU, 2, 5, 9999.0),
        1 => (NW, 2, 3, 9999.0),
        _ => (NH, 3, 1, 340.0),
    }
}
fn card_rect(p: Rect, cols: usize, rows: usize, idx: usize, max_h: f32) -> Rect {
    let top = p.y + 98.0;
    let areah = (p.h - 98.0 - 10.0).min(max_h);
    let gap = 8.0;
    let cw = (p.w - 20.0 - gap * (cols as f32 - 1.0)) / cols as f32;
    let ch = (areah - gap * (rows as f32 - 1.0)) / rows as f32;
    let c = idx % cols;
    let r = idx / cols;
    Rect::new(p.x + 10.0 + c as f32 * (cw + gap), top + r as f32 * (ch + gap), cw, ch)
}

// =====================================================================
//  game state
// =====================================================================

struct Game {
    projs: Vec<Proj>,
    eprojs: Vec<EProj>,
    enemies: Vec<Enemy>,
    parts: Vec<Particle>,
    ftexts: Vec<FText>,
    bolts: Vec<Bolt>,
    rings: Vec<Ring>,
    meteors: Vec<Meteor>,
    gold: u64,
    wave: u32,
    kills: u32,
    best: u32,
    to_spawn: u32,
    spawn_t: f32,
    break_t: f32,
    boss_pending: bool,
    cleared: bool,
    next_id: u32,
    hp: f32,
    over: bool,
    over_t: f32,
    up: [u32; NU],
    owned_w: [bool; NW],
    owned_h: [bool; NH],
    weapon: usize,
    hero: usize,
    sk_cd: [f32; 3],
    rage: f32,
    shoot_cd: f32,
    recoil: f32,
    auto: bool,
    aim: Vec2,
    aim_pt: Vec2,
    muzzle: f32,
    combo: u32,
    combo_t: f32,
    t: f32,
    lw: f32,
    shake: f32,
    flash: f32,
    cold: f32,
    banner_t: f32,
    shop: bool,
    tab: usize,
    msg: String,
    msg_t: f32,
}

impl Game {
    fn new() -> Self {
        let mut g = Game {
            projs: vec![],
            eprojs: vec![],
            enemies: vec![],
            parts: vec![],
            ftexts: vec![],
            bolts: vec![],
            rings: vec![],
            meteors: vec![],
            gold: 0,
            wave: 1,
            kills: 0,
            best: 1,
            to_spawn: 0,
            spawn_t: 1.0,
            break_t: 3.0,
            boss_pending: false,
            cleared: false,
            next_id: 1,
            hp: 1.0,
            over: false,
            over_t: 0.0,
            up: [0; NU],
            owned_w: [true, false, false, false, false],
            owned_h: [true, false, false],
            weapon: 0,
            hero: 0,
            sk_cd: [0.0; 3],
            rage: 0.0,
            shoot_cd: 0.0,
            recoil: 0.0,
            auto: true,
            aim: vec2(0.97, -0.24).normalize(),
            aim_pt: vec2(500.0, 300.0),
            muzzle: 0.0,
            combo: 0,
            combo_t: 0.0,
            t: 0.0,
            lw: 960.0,
            shake: 0.0,
            flash: 0.0,
            cold: 0.0,
            banner_t: 3.0,
            shop: false,
            tab: 0,
            msg: String::new(),
            msg_t: 0.0,
        };
        g.hp = g.hp_max();
        g.start_wave();
        g
    }

    fn restart(&mut self) {
        let lw = self.lw;
        let best = self.best;
        *self = Game::new();
        self.lw = lw;
        self.best = best;
    }

    // ----- stats -----

    fn hp_max(&self) -> f32 {
        let b = 200.0 * (1.0 + 0.25 * self.up[6] as f32);
        if self.hero == 1 { b * 1.6 } else { b }
    }
    fn dmg(&self) -> f32 {
        let w = weapon(self.weapon);
        let mut d = w.dmg * 1.2f32.powi(self.up[0] as i32);
        if self.hero == 1 {
            d *= 1.1;
        }
        if self.rage > 0.0 {
            d *= 1.5;
        }
        d
    }
    fn cooldown(&self) -> f32 {
        let w = weapon(self.weapon);
        let mut c = w.cd / (1.0 + 0.1 * self.up[1] as f32);
        if self.rage > 0.0 {
            c *= 0.4;
        }
        c.max(0.045)
    }
    fn shots(&self) -> u32 {
        1 + self.up[2]
    }
    fn crit_chance(&self) -> f32 {
        let h = if self.hero == 0 { 0.10 } else { 0.0 };
        (0.05 + 0.04 * self.up[3] as f32 + h).min(0.85)
    }
    fn crit_mult(&self) -> f32 {
        2.0 + 0.15 * self.up[4] as f32
    }
    fn gold_mult(&self) -> f32 {
        let h = if self.hero == 2 { 1.25 } else { 1.0 };
        (1.0 + 0.1 * self.up[8] as f32) * h
    }
    fn skill_pow(&self) -> f32 {
        1.0 + 0.1 * self.up[9] as f32
    }
    fn skill_cd(&self, i: usize) -> f32 {
        let b = [18.0f32, 26.0, 32.0];
        let h = if self.hero == 2 { 0.8 } else { 1.0 };
        b[i] * (1.0 - 0.04 * self.up[9] as f32).max(0.5) * h
    }

    fn up_now(&self, i: usize) -> String {
        let l = self.up[i] as f32;
        match i {
            0 => format!("x{:.1}", 1.2f32.powi(self.up[0] as i32)),
            1 => format!("x{:.1}", 1.0 + 0.1 * l),
            2 => format!("{} shots", 1 + self.up[2]),
            3 => format!("{:.0}%", self.crit_chance() * 100.0),
            4 => format!("x{:.2}", self.crit_mult()),
            5 => format!("+{}", self.up[5]),
            6 => format!("{} HP", fmt(self.hp_max() as f64)),
            7 => format!("{} HP/s", self.up[7]),
            8 => format!("+{:.0}%", l * 10.0),
            _ => format!("x{:.1}", self.skill_pow()),
        }
    }

    fn say(&mut self, s: &str) {
        self.msg = s.to_string();
        self.msg_t = 1.6;
    }

    // ----- waves -----

    fn start_wave(&mut self) {
        self.to_spawn = (12 + 4 * self.wave).min(220);
        self.boss_pending = self.wave % 10 == 0;
        self.banner_t = 3.0;
        self.spawn_t = 1.0;
        self.cleared = false;
        if self.wave > self.best {
            self.best = self.wave;
        }
    }

    fn pick_kind(&self) -> usize {
        let mut total = 0u32;
        for &(_, mw, wt) in SPAWN.iter() {
            if self.wave >= mw {
                total += wt;
            }
        }
        let mut r: u32 = rand::gen_range(0u32, total.max(1));
        for &(k, mw, wt) in SPAWN.iter() {
            if self.wave >= mw {
                if r < wt {
                    return k;
                }
                r -= wt;
            }
        }
        0
    }

    fn spawn_kind(&mut self, k: usize, x: f32) {
        let kd = kind(k);
        let w = self.wave as i32;
        let hpm = 1.12f32.powi(w - 1);
        let dm = 1.06f32.powi(w - 1);
        let elite = !kd.boss && k != 11 && self.wave >= 3 && rnd(0.0, 1.0) < 0.05;
        let mut hp = kd.hp * hpm;
        if elite {
            hp *= 3.0;
        }
        let fy = if kd.boss { 165.0 } else { rnd(110.0, 300.0) };
        let id = self.next_id;
        self.next_id += 1;
        self.enemies.push(Enemy {
            id,
            kind: k,
            x,
            hp,
            max_hp: hp,
            hit: 0.0,
            lane: rnd(0.0, 60.0),
            yo: rnd(0.0, 16.0),
            fy,
            ph: rnd(0.0, 6.28),
            atk: 0.6,
            spec: rnd(1.0, 4.0),
            slow: 0.0,
            elite,
            norew: false,
            dm,
        });
    }

    fn wave_logic(&mut self, dt: f32) {
        if self.to_spawn > 0 || self.boss_pending {
            self.spawn_t -= dt;
            if self.spawn_t <= 0.0 {
                if self.boss_pending {
                    let k = 12 + (((self.wave / 10) as usize).max(1) - 1) % 3;
                    let x = self.lw + 80.0;
                    self.spawn_kind(k, x);
                    self.boss_pending = false;
                    self.shake = 0.6;
                    self.spawn_t = 1.2;
                } else if self.enemies.len() < 130 {
                    let k = self.pick_kind();
                    let x = self.lw + 30.0 + rnd(0.0, 160.0);
                    self.spawn_kind(k, x);
                    self.to_spawn -= 1;
                    self.spawn_t = rnd(0.5, 1.2) * (0.7 - 0.02 * self.wave as f32).max(0.05);
                }
            }
        } else if self.enemies.is_empty() {
            if !self.cleared {
                self.cleared = true;
                self.break_t = 3.0;
                let bonus = (15.0 * self.wave as f32 * self.gold_mult()) as u64;
                self.gold += bonus;
                self.hp = (self.hp + self.hp_max() * 0.15).min(self.hp_max());
                self.say(&format!("Wave cleared! +{} gold", fmt(bonus as f64)));
            }
            self.break_t -= dt;
            if self.break_t <= 0.0 {
                self.wave += 1;
                self.start_wave();
            }
        }
    }

    // ----- combat -----

    fn pick_target(&self) -> Option<(Vec2, Vec2)> {
        let mut best: Option<(f32, Vec2, Vec2)> = None;
        for e in self.enemies.iter() {
            if e.hp <= 0.0 || e.x > self.lw + 5.0 {
                continue;
            }
            let rc = enemy_rect(e, self.t);
            let c = vec2(rc.x + rc.w * 0.5, rc.y + rc.h * 0.5);
            let kd = kind(e.kind);
            let sf = if e.slow > 0.0 { 0.25 } else { 1.0 };
            let vx = if e.x > CASTLE_W + 30.0 { -kd.speed * sf } else { 0.0 };
            let score = e.x;
            let better = match best {
                Some((s, _, _)) => score < s,
                None => true,
            };
            if better {
                best = Some((score, c, vec2(vx, 0.0)));
            }
        }
        best.map(|(_, c, v)| (c, v))
    }

    fn hurt(&mut self, idx: usize, dmg: f32, crit: bool, magic: bool, at: Vec2, kb: f32) {
        let mut d = dmg;
        let col;
        {
            let e = &mut self.enemies[idx];
            let kd = kind(e.kind);
            if kd.armor && !magic {
                d *= 0.4;
            }
            e.hp -= d;
            e.hit = 0.08;
            if !kd.boss {
                e.x += kb;
            }
            col = if kd.armor { rgba(0.9, 0.9, 1.0, 1.0) } else { kd.color };
        }
        let n = if crit { 7 } else { 3 };
        burst(&mut self.parts, at, n, col, 130.0, 0.35);
        if crit {
            self.shake = (self.shake + 0.04).min(0.7);
        }
        if self.ftexts.len() < 28 {
            self.ftexts.push(FText { p: at, val: d, crit, life: 0.8 });
        }
    }

    fn explode(&mut self, at: Vec2, r: f32, dmg: f32, magic: bool, crit: bool) {
        let t = self.t;
        for i in 0..self.enemies.len() {
            let rc = enemy_rect(&self.enemies[i], t);
            let c = vec2(rc.x + rc.w * 0.5, rc.y + rc.h * 0.5);
            let d = (c - at).length();
            let reach = r + rc.w * 0.5;
            if d < reach {
                let f = 1.0 - 0.4 * (d / reach).min(1.0);
                self.hurt(i, dmg * f, crit, magic, c, 6.0);
            }
        }
        self.rings.push(Ring { p: at, r, life: 0.35, max: 0.35, c: rgba(1.0, 0.6, 0.2, 1.0) });
        burst(&mut self.parts, at, 16, rgba(1.0, 0.55, 0.15, 1.0), 260.0, 0.6);
        burst(&mut self.parts, at, 8, rgba(1.0, 0.9, 0.5, 1.0), 160.0, 0.45);
        self.shake = (self.shake + 0.08 + r * 0.001).min(0.7);
    }

    fn lightning(&mut self, from: Vec2, dmg: f32, chain: u32, crit: bool) {
        let t = self.t;
        let mut cur: Option<usize> = None;
        let mut best = 260.0f32;
        for (i, e) in self.enemies.iter().enumerate() {
            if e.hp <= 0.0 {
                continue;
            }
            let rc = enemy_rect(e, t);
            let c = vec2(rc.x + rc.w * 0.5, rc.y + rc.h * 0.5);
            let d = (c - self.aim_pt).length();
            if d < best {
                best = d;
                cur = Some(i);
            }
        }
        let mut hit: Vec<usize> = Vec::new();
        let mut pts: Vec<Vec2> = vec![from];
        let mut n = 0u32;
        while let Some(i) = cur {
            if n >= chain {
                break;
            }
            n += 1;
            hit.push(i);
            let rc = enemy_rect(&self.enemies[i], t);
            let c = vec2(rc.x + rc.w * 0.5, rc.y + rc.h * 0.5);
            pts.push(c);
            self.hurt(i, dmg * 0.93f32.powi(n as i32 - 1), crit, true, c, 0.0);
            let mut nx: Option<usize> = None;
            let mut bd = 190.0f32;
            for (j, e) in self.enemies.iter().enumerate() {
                if e.hp <= 0.0 || hit.contains(&j) {
                    continue;
                }
                let r2 = enemy_rect(e, t);
                let c2 = vec2(r2.x + r2.w * 0.5, r2.y + r2.h * 0.5);
                let d = (c2 - c).length();
                if d < bd {
                    bd = d;
                    nx = Some(j);
                }
            }
            cur = nx;
        }
        if pts.len() == 1 {
            pts.push(from + self.aim * 420.0);
        }
        let j = jagged(&pts);
        self.bolts.push(Bolt { pts: j, life: 0.2, c: rgba(0.5, 0.85, 1.0, 1.0) });
        burst(&mut self.parts, from, 5, rgba(0.6, 0.9, 1.0, 1.0), 140.0, 0.3);
    }

    fn fire(&mut self) {
        let w = weapon(self.weapon);
        let o = hero_origin();
        let n = self.shots();
        let dir = self.aim;
        let base_d = self.dmg();
        let cc = self.crit_chance();
        let cm = self.crit_mult();
        let tip = o + dir * 36.0;
        if w.kind == 3 {
            let crit = rnd(0.0, 1.0) < cc;
            let d = base_d * if crit { cm } else { 1.0 };
            self.lightning(tip, d, 3 + n, crit);
        } else {
            let step = if w.kind == 4 { 0.12 } else { 0.07 };
            for i in 0..n {
                let off = (i as f32 - (n as f32 - 1.0) / 2.0) * step;
                let (sn, cs) = off.sin_cos();
                let dd = vec2(dir.x * cs - dir.y * sn, dir.x * sn + dir.y * cs);
                let crit = rnd(0.0, 1.0) < cc;
                let d = base_d * if crit { cm } else { 1.0 };
                let pierce = if w.splash > 0.0 { 1 } else { w.pierce + self.up[5] };
                self.projs.push(Proj {
                    p: tip,
                    v: dd * w.speed,
                    dmg: d,
                    crit,
                    kind: w.kind,
                    pierce,
                    splash: w.splash,
                    grav: w.grav,
                    magic: w.magic,
                    alive: true,
                    hit: Vec::new(),
                    life: 3.0,
                });
            }
        }
        self.recoil = 1.0;
        self.muzzle = 1.0;
        let col = match w.kind {
            2 => rgba(1.0, 0.55, 0.15, 1.0),
            3 => rgba(0.55, 0.9, 1.0, 1.0),
            _ => rgba(1.0, 0.9, 0.5, 1.0),
        };
        burst(&mut self.parts, tip, 4, col, 150.0, 0.25);
        if w.kind == 4 {
            self.shake = (self.shake + 0.25).min(0.8);
            burst(&mut self.parts, tip, 8, rgba(0.6, 0.6, 0.65, 1.0), 120.0, 0.7);
        }
    }

    fn castle_hit(&mut self, d: f32) {
        self.hp -= d;
        self.flash = (self.flash + 0.25).min(0.6);
        self.shake = (self.shake + 0.06).min(0.7);
        for _ in 0..2 {
            let y = GROUND - rnd(20.0, 120.0);
            burst(&mut self.parts, vec2(CASTLE_W, y), 2, rgba(0.6, 0.6, 0.68, 1.0), 110.0, 0.5);
        }
        if self.hp <= 0.0 {
            self.hp = 0.0;
            self.over = true;
            self.over_t = 0.0;
            if self.wave > self.best {
                self.best = self.wave;
            }
        }
    }

    fn cast(&mut self, i: usize) {
        if self.sk_cd[i] > 0.0 {
            return;
        }
        self.sk_cd[i] = self.skill_cd(i);
        match i {
            0 => {
                for _ in 0..16 {
                    let x = rnd(self.lw * 0.2, self.lw);
                    let y = rnd(-700.0, -40.0);
                    self.meteors.push(Meteor { p: vec2(x, y), v: vec2(-180.0, 620.0), alive: true });
                }
            }
            1 => {
                let d = 3.0 * self.dmg() * self.skill_pow();
                let t = self.t;
                for k in 0..self.enemies.len() {
                    let rc = enemy_rect(&self.enemies[k], t);
                    let c = vec2(rc.x + rc.w * 0.5, rc.y + rc.h * 0.5);
                    self.enemies[k].slow = 5.0;
                    self.hurt(k, d, false, true, c, 0.0);
                }
                self.cold = 1.0;
                self.rings.push(Ring { p: vec2(CASTLE_W, GROUND - 60.0), r: self.lw, life: 0.7, max: 0.7, c: rgba(0.6, 0.9, 1.0, 1.0) });
            }
            _ => {
                self.rage = 8.0;
                self.flash = 0.0;
                self.rings.push(Ring { p: hero_origin(), r: 120.0, life: 0.5, max: 0.5, c: rgba(1.0, 0.4, 0.2, 1.0) });
            }
        }
    }

    fn reap(&mut self) {
        let t = self.t;
        let all = std::mem::take(&mut self.enemies);
        let mut keep: Vec<Enemy> = Vec::with_capacity(all.len());
        let mut dead: Vec<Enemy> = Vec::new();
        for e in all {
            if e.hp <= 0.0 {
                dead.push(e);
            } else {
                keep.push(e);
            }
        }
        self.enemies = keep;
        for e in dead {
            let kd = kind(e.kind);
            let rc = enemy_rect(&e, t);
            let c = vec2(rc.x + rc.w * 0.5, rc.y + rc.h * 0.5);
            self.kills += 1;
            if !e.norew {
                self.combo += 1;
                self.combo_t = 2.2;
                let cb = 1.0 + (self.combo.min(50)) as f32 * 0.01;
                let rm = 1.10f32.powi(self.wave as i32 - 1);
                let el = if e.elite { 4.0 } else { 1.0 };
                let r = kd.reward * rm * el * self.gold_mult() * cb;
                self.gold += (r as u64).max(1);
            }
            let big = kd.boss;
            let n = if big { 50 } else { 7 };
            let sp = if big { 300.0 } else { 140.0 };
            burst(&mut self.parts, c, n, kd.color, sp, 0.6);
            burst(&mut self.parts, c, if big { 20 } else { 2 }, rgba(1.0, 0.85, 0.2, 1.0), 160.0, 0.8);
            if big {
                self.shake = (self.shake + 0.6).min(0.9);
                self.rings.push(Ring { p: c, r: 200.0, life: 0.6, max: 0.6, c: rgba(1.0, 0.9, 0.5, 1.0) });
            }
            if kd.splits && !e.norew {
                self.spawn_kind(11, e.x - 8.0);
                self.spawn_kind(11, e.x + 8.0);
            }
        }
    }

    fn update_fx(&mut self, dt: f32) {
        for p in self.parts.iter_mut() {
            p.life -= dt;
            p.v.y += p.grav * dt;
            p.p += p.v * dt;
        }
        self.parts.retain(|p| p.life > 0.0);
        for f in self.ftexts.iter_mut() {
            f.life -= dt;
            f.p.y -= 40.0 * dt;
        }
        self.ftexts.retain(|f| f.life > 0.0);
        for b in self.bolts.iter_mut() {
            b.life -= dt;
        }
        self.bolts.retain(|b| b.life > 0.0);
        for r in self.rings.iter_mut() {
            r.life -= dt;
        }
        self.rings.retain(|r| r.life > 0.0);
    }

    // ----- shop -----

    fn shop_buy(&mut self, i: usize) {
        match self.tab {
            0 => {
                let lvl = self.up[i];
                if lvl >= UP_MAX[i] {
                    self.say("Max level");
                    return;
                }
                let c = up_cost(i, lvl);
                if self.gold < c {
                    self.say("Not enough gold");
                    return;
                }
                self.gold -= c;
                let before = self.hp_max();
                self.up[i] += 1;
                if i == 6 {
                    self.hp += self.hp_max() - before;
                }
                self.say("Upgraded!");
            }
            1 => {
                if self.owned_w[i] {
                    self.weapon = i;
                    self.say("Equipped");
                } else if self.gold >= weapon(i).price {
                    self.gold -= weapon(i).price;
                    self.owned_w[i] = true;
                    self.weapon = i;
                    self.say("Bought and equipped!");
                } else {
                    self.say("Not enough gold");
                }
            }
            _ => {
                let before = self.hp_max();
                let ratio = self.hp / before;
                if self.owned_h[i] {
                    self.hero = i;
                    self.hp = ratio * self.hp_max();
                    self.say("Hero selected");
                } else if self.gold >= hero_info(i).2 {
                    self.gold -= hero_info(i).2;
                    self.owned_h[i] = true;
                    self.hero = i;
                    self.hp = ratio * self.hp_max();
                    self.say("Hero unlocked!");
                } else {
                    self.say("Not enough gold");
                }
            }
        }
    }

    fn shop_input(&mut self, pressed: bool, mp: Vec2) {
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::S) {
            self.shop = false;
            return;
        }
        if !pressed {
            return;
        }
        let p = shop_panel(self.lw);
        if close_rect(p).contains(mp) || !p.contains(mp) {
            self.shop = false;
            return;
        }
        for i in 0..3 {
            if tab_rect(p, i).contains(mp) {
                self.tab = i;
                return;
            }
        }
        let (n, cols, rows, mh) = tab_grid(self.tab);
        for i in 0..n {
            if card_rect(p, cols, rows, i, mh).contains(mp) {
                self.shop_buy(i);
                return;
            }
        }
    }

    // ----- main update -----

    fn update(&mut self, dt: f32, u: f32) {
        let (mx, my) = mouse_position();
        let mp = vec2(mx / u, my / u);
        let pressed = is_mouse_button_pressed(MouseButton::Left);
        let down = is_mouse_button_down(MouseButton::Left);

        self.t += dt;
        self.msg_t = (self.msg_t - dt).max(0.0);
        self.flash = (self.flash - dt * 1.5).max(0.0);
        self.cold = (self.cold - dt * 0.8).max(0.0);
        self.shake = (self.shake - dt * 1.6).max(0.0);
        self.banner_t = (self.banner_t - dt).max(0.0);
        self.recoil = (self.recoil - dt * 6.0).max(0.0);
        self.muzzle = (self.muzzle - dt * 10.0).max(0.0);

        if self.over {
            self.over_t += dt;
            self.update_fx(dt);
            if pressed && self.over_t > 0.7 {
                self.restart();
            }
            return;
        }

        if self.shop {
            self.shop_input(pressed, mp);
            return;
        }

        // --- buttons ---
        if pressed {
            if btn_shop().contains(mp) {
                self.shop = true;
                return;
            }
            if btn_auto().contains(mp) {
                self.auto = !self.auto;
            }
            for i in 0..3 {
                if btn_skill(self.lw, i).contains(mp) {
                    self.cast(i);
                }
            }
        }
        if is_key_pressed(KeyCode::Key1) {
            self.cast(0);
        }
        if is_key_pressed(KeyCode::Key2) {
            self.cast(1);
        }
        if is_key_pressed(KeyCode::Key3) {
            self.cast(2);
        }
        if is_key_pressed(KeyCode::S) {
            self.shop = true;
            return;
        }
        if is_key_pressed(KeyCode::A) {
            self.auto = !self.auto;
        }

        // --- timers ---
        for i in 0..3 {
            self.sk_cd[i] = (self.sk_cd[i] - dt).max(0.0);
        }
        self.rage = (self.rage - dt).max(0.0);
        if self.combo_t > 0.0 {
            self.combo_t -= dt;
            if self.combo_t <= 0.0 {
                self.combo = 0;
            }
        }
        let regen = self.up[7] as f32;
        if regen > 0.0 {
            self.hp = (self.hp + regen * dt).min(self.hp_max());
        }

        // --- aiming and shooting ---
        let w = weapon(self.weapon);
        let o = hero_origin();
        let touching = down && mp.y < UI_TOP - 4.0;
        let mut aim_from: Option<(Vec2, Vec2)> = None;
        if touching {
            aim_from = Some((mp, vec2(0.0, 0.0)));
        } else if self.auto {
            aim_from = self.pick_target();
        }
        if let Some((tp, tv)) = aim_from {
            let sp = if w.speed > 0.0 { w.speed } else { 20000.0 };
            self.aim = solve_aim(o, tp, tv, sp, w.grav);
            self.aim_pt = tp;
        }
        self.shoot_cd -= dt;
        if aim_from.is_some() && self.shoot_cd <= 0.0 {
            self.fire();
            self.shoot_cd = self.cooldown();
        }

        // --- player projectiles ---
        let t = self.t;
        let mut projs = std::mem::take(&mut self.projs);
        for p in projs.iter_mut() {
            p.life -= dt;
            if p.life <= 0.0 {
                p.alive = false;
                continue;
            }
            let dist = p.v.length() * dt;
            let steps = ((dist / 10.0).ceil() as i32).max(1);
            let sdt = dt / steps as f32;
            'steps: for _ in 0..steps {
                p.v.y += p.grav * sdt;
                p.p += p.v * sdt;
                if p.p.y > GROUND + 12.0 || p.p.x > self.lw + 250.0 || p.p.x < -80.0 || p.p.y < -400.0 {
                    if p.splash > 0.0 && p.p.y > GROUND && p.p.x < self.lw + 50.0 {
                        let at = vec2(p.p.x, GROUND);
                        self.explode(at, p.splash, p.dmg, p.magic, p.crit);
                    }
                    p.alive = false;
                    break 'steps;
                }
                let mut hit_idx: Option<usize> = None;
                for (i, e) in self.enemies.iter().enumerate() {
                    if e.hp <= 0.0 || p.hit.contains(&e.id) {
                        continue;
                    }
                    let rc = enemy_rect(e, t);
                    let big = Rect::new(rc.x - 3.0, rc.y - 3.0, rc.w + 6.0, rc.h + 6.0);
                    if big.contains(p.p) {
                        hit_idx = Some(i);
                        break;
                    }
                }
                if let Some(i) = hit_idx {
                    let id = self.enemies[i].id;
                    p.hit.push(id);
                    let at = p.p;
                    if p.splash > 0.0 {
                        self.explode(at, p.splash, p.dmg, p.magic, p.crit);
                        p.alive = false;
                        break 'steps;
                    }
                    self.hurt(i, p.dmg, p.crit, p.magic, at, 4.0);
                    if p.pierce <= 1 {
                        p.alive = false;
                        break 'steps;
                    }
                    p.pierce -= 1;
                }
            }
            if p.alive && self.parts.len() < MAX_PARTS {
                if p.kind == 2 {
                    burst(&mut self.parts, p.p, 1, rgba(1.0, 0.5, 0.1, 1.0), 40.0, 0.3);
                } else if p.kind == 4 {
                    burst(&mut self.parts, p.p, 1, rgba(0.5, 0.5, 0.55, 1.0), 25.0, 0.5);
                }
            }
        }
        projs.retain(|p| p.alive);
        self.projs = projs;

        // --- meteors ---
        let mut ms = std::mem::take(&mut self.meteors);
        let mut impacts: Vec<Vec2> = Vec::new();
        for m in ms.iter_mut() {
            m.p += m.v * dt;
            if m.p.y >= GROUND + 4.0 {
                m.alive = false;
                impacts.push(vec2(m.p.x, GROUND));
            }
        }
        ms.retain(|m| m.alive);
        self.meteors = ms;
        let md = 5.0 * self.dmg() * self.skill_pow();
        for at in impacts {
            self.explode(at, 100.0, md, true, false);
        }

        // --- enemy projectiles ---
        let mut ep_dmg = 0.0f32;
        for ep in self.eprojs.iter_mut() {
            ep.p += ep.v * dt;
            if ep.p.x <= CASTLE_W - 4.0 {
                ep.alive = false;
                ep_dmg += ep.dmg;
            }
        }
        self.eprojs.retain(|e| e.alive);
        if ep_dmg > 0.0 {
            self.castle_hit(ep_dmg);
        }

        // --- enemies ---
        let mut enemies = std::mem::take(&mut self.enemies);
        let mut castle_dmg = 0.0f32;
        let mut new_ep: Vec<EProj> = Vec::new();
        let mut spawns: Vec<(usize, f32)> = Vec::new();
        let mut heals: Vec<f32> = Vec::new();
        let mut booms: Vec<Vec2> = Vec::new();
        let mut stomps = 0u32;
        for e in enemies.iter_mut() {
            let kd = kind(e.kind);
            e.hit = (e.hit - dt).max(0.0);
            let sf = if e.slow > 0.0 { 0.25 } else { 1.0 };
            e.slow = (e.slow - dt).max(0.0);
            let stop_x = if kd.range > 0.0 {
                CASTLE_W + kd.range + e.lane
            } else {
                CASTLE_W + kd.w * 0.5 + e.lane * 0.3
            };
            if e.x > stop_x {
                e.x -= kd.speed * sf * dt;
            } else {
                e.atk -= dt * if sf < 1.0 { 0.5 } else { 1.0 };
                if e.atk <= 0.0 {
                    let rc = enemy_rect(e, t);
                    let cy = rc.y + rc.h * 0.5;
                    if kd.bomb {
                        castle_dmg += kd.dmg * e.dm;
                        booms.push(vec2(e.x, cy));
                        e.hp = 0.0;
                        e.norew = true;
                    } else if kd.heal {
                        e.atk = 5.0;
                    } else if kd.range > 0.0 {
                        let from = vec2(e.x - kd.w * 0.3, cy);
                        let target = vec2(CASTLE_W - 8.0, GROUND - rnd(30.0, 120.0));
                        let d = (target - from).normalize();
                        let sp = if kd.boss { 330.0 } else { 260.0 };
                        let k = if kd.boss && kd.flying { 1 } else if kd.summon { 2 } else { 0 };
                        new_ep.push(EProj { p: from, v: d * sp, dmg: kd.dmg * e.dm, kind: k, alive: true });
                        e.atk = if kd.boss { 1.8 } else { 2.4 };
                    } else {
                        castle_dmg += kd.dmg * e.dm;
                        e.atk = 1.0;
                    }
                }
            }
            if kd.summon {
                e.spec -= dt;
                if e.spec <= 0.0 {
                    e.spec = 6.0;
                    for k in 0..3 {
                        spawns.push((4, e.x + 24.0 + k as f32 * 20.0));
                    }
                }
            }
            if kd.heal {
                e.spec -= dt;
                if e.spec <= 0.0 {
                    e.spec = 2.2;
                    heals.push(e.x);
                }
            }
            if e.kind == 12 && e.x <= stop_x + 6.0 {
                e.spec -= dt;
                if e.spec <= 0.0 {
                    e.spec = 6.0;
                    stomps += 1;
                    castle_dmg += kd.dmg * 0.8 * e.dm;
                }
            }
        }
        for hx in heals {
            for e in enemies.iter_mut() {
                if e.hp > 0.0 && (e.x - hx).abs() < 150.0 {
                    e.hp = (e.hp + e.max_hp * 0.10).min(e.max_hp);
                }
            }
            burst(&mut self.parts, vec2(hx, GROUND - 30.0), 6, rgba(0.3, 1.0, 0.5, 1.0), 90.0, 0.6);
            self.rings.push(Ring { p: vec2(hx, GROUND - 20.0), r: 150.0, life: 0.5, max: 0.5, c: rgba(0.3, 1.0, 0.5, 1.0) });
        }
        self.enemies = enemies;
        self.eprojs.extend(new_ep);
        for (k, x) in spawns {
            self.spawn_kind(k, x);
        }
        for b in booms {
            burst(&mut self.parts, b, 18, rgba(1.0, 0.5, 0.1, 1.0), 260.0, 0.6);
            self.rings.push(Ring { p: b, r: 90.0, life: 0.35, max: 0.35, c: rgba(1.0, 0.5, 0.1, 1.0) });
            self.shake = (self.shake + 0.25).min(0.8);
        }
        if stomps > 0 {
            self.shake = (self.shake + 0.4).min(0.9);
        }
        if castle_dmg > 0.0 {
            self.castle_hit(castle_dmg);
        }

        self.reap();
        self.update_fx(dt);
        self.wave_logic(dt);
    }
}

// =====================================================================
//  drawing: world
// =====================================================================

type Pal = (Color, Color, Color);

fn draw_sky(g: &G, lw: f32, t: f32, b: &Biome) {
    let n = 30;
    let hh = GROUND / n as f32;
    for i in 0..n {
        let f = i as f32 / (n as f32 - 1.0);
        g.rect(0.0, i as f32 * hh, lw + 2.0, hh + 1.0, mix(b.top, b.bot, f * f));
    }
    // stars
    for i in 0..60 {
        let x = (i as f32 * 97.3) % lw;
        let y = (i as f32 * 53.7) % (GROUND * 0.6);
        let a = 0.35 + 0.5 * (t * 2.0 + i as f32).sin().abs();
        g.rect(x, y, 2.0, 2.0, rgba(1.0, 1.0, 1.0, a * 0.8));
    }
    // moon / sun
    let (mx, my, mr) = if b.amb == 1 { (lw * 0.78, 250.0, 44.0) } else if b.amb == 3 { (lw * 0.8, 110.0, 50.0) } else { (lw * 0.82, 90.0, 28.0) };
    g.glow(mx, my, mr * 2.2, with_a(b.moon, 0.5));
    g.circ(mx, my, mr, b.moon);
    if b.amb == 0 || b.amb == 2 || b.amb == 4 {
        let dk = shade(b.moon, 0.85);
        g.circ(mx - 8.0, my - 6.0, 6.0, dk);
        g.circ(mx + 9.0, my + 8.0, 4.0, dk);
        g.circ(mx + 6.0, my - 10.0, 3.0, dk);
    }
    // clouds
    for i in 0..5 {
        let x = ((i as f32 * 260.0 + t * (6.0 + i as f32 * 2.0)) % (lw + 300.0)) - 150.0;
        let y = 40.0 + i as f32 * 28.0;
        let c = with_a(mix(b.bot, WHITE, 0.4), 0.14);
        g.circ(x, y, 18.0, c);
        g.circ(x + 20.0, y - 6.0, 22.0, c);
        g.circ(x + 42.0, y, 18.0, c);
        g.rect(x, y, 42.0, 18.0, c);
    }
    // far mountains
    let count = (lw / 120.0) as i32 + 3;
    for i in 0..count {
        let x = i as f32 * 120.0 - 60.0;
        let h = 80.0 + ((i as f32 * 1.7).sin() * 0.5 + 0.5) * 80.0;
        g.tri((x, GROUND), (x + 60.0, GROUND - h), (x + 120.0, GROUND), b.far);
        if b.amb == 2 {
            g.tri((x + 48.0, GROUND - h + 18.0), (x + 60.0, GROUND - h), (x + 72.0, GROUND - h + 18.0), rgba(1.0, 1.0, 1.0, 0.8));
        }
        if b.amb == 3 && i % 3 == 0 {
            g.glow(x + 60.0, GROUND - h + 4.0, 20.0, rgba(1.0, 0.4, 0.1, 0.8));
        }
    }
    // near hills
    let count2 = (lw / 90.0) as i32 + 3;
    for i in 0..count2 {
        let x = i as f32 * 90.0 - 30.0;
        let h = 35.0 + ((i as f32 * 2.3 + 1.0).sin() * 0.5 + 0.5) * 45.0;
        g.tri((x, GROUND), (x + 45.0, GROUND - h), (x + 90.0, GROUND), b.near);
    }
}

fn draw_ambient(g: &G, lw: f32, t: f32, b: &Biome) {
    for i in 0..70 {
        let seed = i as f32 * 12.9898;
        let fx = (seed.sin() * 43758.547).fract().abs();
        let fy = ((seed * 1.7).sin() * 24634.635).fract().abs();
        match b.amb {
            1 => {
                let x = (fx * lw + t * (40.0 + fx * 60.0)) % lw;
                let y = GROUND - 10.0 - fy * 150.0;
                g.rect(x, y, 3.0, 1.5, rgba(0.95, 0.8, 0.5, 0.35));
            }
            2 => {
                let x = (fx * lw + (t * 0.8 + seed).sin() * 18.0).rem_euclid(lw);
                let y = (fy * GROUND + t * (25.0 + fx * 40.0)) % GROUND;
                g.circ(x, y, 1.0 + fx * 1.6, rgba(1.0, 1.0, 1.0, 0.8));
            }
            3 => {
                let x = (fx * lw + (t + seed).sin() * 20.0).rem_euclid(lw);
                let h = (fy * GROUND + t * (30.0 + fx * 50.0)) % GROUND;
                let y = GROUND - h;
                let a = 1.0 - h / GROUND;
                g.circ(x, y, 1.2 + fx * 1.5, rgba(1.0, 0.55, 0.15, a));
            }
            4 => {
                let x = fx * lw + (t * 0.5 + seed).sin() * 30.0;
                let y = fy * GROUND + (t * 0.4 + seed).cos() * 20.0;
                let a = 0.3 + 0.4 * (t + seed).sin().abs();
                g.circ(x, y, 1.5 + fx * 1.5, rgba(0.8, 0.5, 1.0, a));
            }
            _ => {
                if i < 20 {
                    let x = fx * lw + (t * 0.6 + seed).sin() * 25.0;
                    let y = GROUND - 25.0 - fy * 110.0 + (t * 0.9 + seed).cos() * 12.0;
                    let a = 0.4 + 0.5 * (t * 2.0 + seed).sin().abs();
                    g.glow(x, y, 6.0, rgba(0.8, 1.0, 0.4, a));
                    g.circ(x, y, 1.5, rgba(1.0, 1.0, 0.7, a));
                }
            }
        }
    }
}

fn draw_ground(g: &G, lw: f32, t: f32, b: &Biome) {
    let w = lw + 40.0;
    g.rect(-20.0, GROUND - 2.0, w, 22.0, b.grass);
    g.rect(-20.0, GROUND + 18.0, w, UI_TOP - GROUND - 18.0 + 8.0, b.dirt);
    g.rect(-20.0, GROUND - 2.0, w, 3.0, b.tuft);
    let n = (lw / 12.0) as i32 + 3;
    for i in 0..n {
        let x = i as f32 * 12.0;
        let h = 4.0 + ((i * 7) % 5) as f32;
        g.tri((x, GROUND - 2.0), (x + 4.0, GROUND - 2.0 - h), (x + 8.0, GROUND - 2.0), b.tuft);
    }
    // decorations
    let m = (lw / 150.0) as i32 + 2;
    for i in 1..m {
        let x = i as f32 * 150.0 + ((i * 37) % 60) as f32;
        if x < CASTLE_W + 40.0 {
            continue;
        }
        let y = GROUND + 6.0 + ((i * 13) % 10) as f32;
        match b.amb {
            1 => {
                let c = rgba(0.25, 0.5, 0.25, 1.0);
                g.rect(x - 3.0, y - 28.0, 6.0, 28.0, c);
                g.rect(x - 11.0, y - 20.0, 4.0, 10.0, c);
                g.rect(x - 11.0, y - 20.0, 9.0, 3.0, c);
                g.rect(x + 7.0, y - 24.0, 4.0, 10.0, c);
                g.rect(x + 2.0, y - 24.0, 9.0, 3.0, c);
            }
            2 => {
                g.rect(x - 2.0, y - 8.0, 4.0, 8.0, rgba(0.3, 0.2, 0.12, 1.0));
                g.tri((x - 14.0, y - 8.0), (x, y - 36.0), (x + 14.0, y - 8.0), rgba(0.15, 0.4, 0.3, 1.0));
                g.tri((x - 9.0, y - 22.0), (x, y - 40.0), (x + 9.0, y - 22.0), rgba(0.2, 0.5, 0.38, 1.0));
                g.tri((x - 5.0, y - 33.0), (x, y - 42.0), (x + 5.0, y - 33.0), rgba(1.0, 1.0, 1.0, 0.9));
            }
            3 => {
                let a = 0.6 + 0.3 * (t * 3.0 + i as f32).sin();
                g.glow(x, y, 16.0, rgba(1.0, 0.4, 0.1, a));
                g.rect(x - 14.0, y - 1.0, 28.0, 3.0, rgba(1.0, 0.55, 0.15, a));
                g.rect(x - 6.0, y - 3.0, 3.0, 3.0, rgba(1.0, 0.8, 0.3, a));
            }
            4 => {
                let a = 0.5 + 0.3 * (t * 2.0 + i as f32).sin();
                g.glow(x, y - 12.0, 18.0, rgba(0.7, 0.4, 1.0, a));
                g.tri((x - 6.0, y), (x, y - 28.0), (x + 6.0, y), rgba(0.65, 0.4, 0.95, 1.0));
                g.tri((x + 3.0, y), (x + 9.0, y - 18.0), (x + 14.0, y), rgba(0.5, 0.3, 0.8, 1.0));
            }
            _ => {
                g.circ(x, y - 6.0, 9.0, rgba(0.12, 0.3, 0.14, 1.0));
                g.circ(x + 9.0, y - 4.0, 7.0, rgba(0.15, 0.36, 0.17, 1.0));
                g.circ(x - 8.0, y - 3.0, 6.0, rgba(0.13, 0.32, 0.15, 1.0));
            }
        }
    }
}

fn draw_castle(g: &G, t: f32, hp_ratio: f32) {
    let cy = GROUND - CASTLE_H;
    let stone = rgba(0.45, 0.46, 0.54, 1.0);
    let stone_d = rgba(0.30, 0.31, 0.39, 1.0);
    let stone_l = rgba(0.58, 0.59, 0.68, 1.0);
    let wood = rgba(0.32, 0.18, 0.08, 1.0);
    let roof = rgba(0.7, 0.12, 0.16, 1.0);

    // wall
    g.rect(0.0, cy, CASTLE_W, CASTLE_H + 14.0, stone);
    let rows = ((CASTLE_H + 14.0) / 16.0) as i32;
    for r in 0..rows {
        let y = cy + r as f32 * 16.0;
        g.rect(0.0, y, CASTLE_W, 1.5, stone_d);
        let mut x = if r % 2 == 0 { 0.0 } else { 16.0 };
        while x < CASTLE_W {
            g.rect(x, y, 1.5, 16.0, stone_d);
            x += 32.0;
        }
    }
    g.rect(0.0, cy, CASTLE_W, 3.0, stone_l);
    g.rect(CASTLE_W - 8.0, cy, 8.0, CASTLE_H + 14.0, with_a(BLACK, 0.18));

    // battlements between the towers
    for i in 0..3 {
        let x = 40.0 + i as f32 * 18.0;
        g.rect(x, cy - 12.0, 11.0, 12.0, stone_l);
        g.rect(x + 8.0, cy - 12.0, 3.0, 12.0, stone_d);
    }

    // towers
    for tx in [0.0f32, CASTLE_W - 38.0] {
        g.rect(tx, cy - 38.0, 38.0, CASTLE_H + 52.0, mix(stone, stone_l, 0.2));
        g.rect(tx, cy - 38.0, 38.0, 3.0, stone_l);
        g.rect(tx + 30.0, cy - 38.0, 8.0, CASTLE_H + 52.0, with_a(BLACK, 0.15));
        for r in 0..9 {
            g.rect(tx, cy - 22.0 + r as f32 * 16.0, 38.0, 1.5, stone_d);
        }
        g.tri((tx - 5.0, cy - 38.0), (tx + 43.0, cy - 38.0), (tx + 19.0, cy - 80.0), roof);
        g.tri((tx + 19.0, cy - 80.0), (tx + 43.0, cy - 38.0), (tx + 28.0, cy - 38.0), shade(roof, 0.7));
        let glow = 0.7 + 0.3 * (t * 6.0 + tx).sin();
        g.rect(tx + 16.0, cy - 24.0, 6.0, 16.0, rgba(1.0, 0.8, 0.3, glow));
        g.rect(tx + 16.0, cy + 20.0, 6.0, 16.0, rgba(1.0, 0.8, 0.3, glow));
    }
    // flags
    for (k, tx) in [0.0f32, CASTLE_W - 38.0].iter().enumerate() {
        let fx = tx + 19.0;
        g.rect(fx - 1.0, cy - 104.0, 2.5, 26.0, rgba(0.35, 0.2, 0.1, 1.0));
        let wv = (t * 4.0 + k as f32).sin();
        g.tri((fx + 1.0, cy - 104.0), (fx + 1.0, cy - 90.0), (fx + 24.0 + wv * 3.0, cy - 97.0 + wv * 2.0), rgba(0.9, 0.15, 0.2, 1.0));
    }

    // gate
    let gx = CASTLE_W / 2.0;
    g.circ(gx, GROUND - 48.0, 17.0, wood);
    g.rect(gx - 17.0, GROUND - 48.0, 34.0, 48.0 + 14.0, wood);
    g.rect(gx - 1.0, GROUND - 64.0, 2.0, 76.0, rgba(0.16, 0.09, 0.04, 1.0));
    for k in 0..4 {
        g.rect(gx - 17.0, GROUND - 12.0 - k as f32 * 13.0, 34.0, 2.0, rgba(0.16, 0.09, 0.04, 1.0));
    }
    g.circ(gx - 8.0, GROUND - 22.0, 1.8, rgba(0.9, 0.75, 0.3, 1.0));
    g.circ(gx + 8.0, GROUND - 22.0, 1.8, rgba(0.9, 0.75, 0.3, 1.0));

    // banner
    g.rect(gx - 8.0, cy + 8.0, 16.0, 38.0, rgba(0.8, 0.12, 0.18, 1.0));
    g.tri((gx - 8.0, cy + 46.0), (gx + 8.0, cy + 46.0), (gx, cy + 56.0), rgba(0.8, 0.12, 0.18, 1.0));
    g.circ(gx, cy + 24.0, 5.0, rgba(1.0, 0.85, 0.3, 1.0));

    // torches
    for tx in [gx - 28.0, gx + 28.0] {
        let ty = GROUND - 60.0;
        g.rect(tx - 1.5, ty, 3.0, 14.0, rgba(0.35, 0.2, 0.1, 1.0));
        let fl = (t * 12.0 + tx).sin();
        g.glow(tx, ty - 2.0, 20.0, rgba(1.0, 0.6, 0.1, 0.8));
        g.circ(tx, ty - 2.0, 5.0 + fl * 1.2, rgba(1.0, 0.55, 0.1, 1.0));
        g.circ(tx, ty - 4.0, 3.0, rgba(1.0, 0.9, 0.3, 1.0));
    }

    // damage
    let dark = rgba(0.12, 0.12, 0.16, 1.0);
    if hp_ratio < 0.65 {
        g.line(30.0, cy + 40.0, 44.0, cy + 58.0, 2.0, dark);
        g.line(44.0, cy + 58.0, 38.0, cy + 74.0, 2.0, dark);
        g.line(44.0, cy + 58.0, 58.0, cy + 64.0, 2.0, dark);
    }
    if hp_ratio < 0.4 {
        g.line(84.0, cy + 30.0, 74.0, cy + 52.0, 2.5, dark);
        g.line(74.0, cy + 52.0, 88.0, cy + 70.0, 2.5, dark);
        g.line(20.0, cy + 90.0, 34.0, cy + 100.0, 2.5, dark);
        for i in 0..4 {
            let f = (t * 30.0 + i as f32 * 40.0) % 90.0;
            let a = 0.4 * (1.0 - f / 90.0);
            g.circ(30.0 + i as f32 * 22.0 + (t * 2.0 + i as f32).sin() * 5.0, cy - 12.0 - f, 6.0 + f * 0.08, rgba(0.3, 0.3, 0.33, a));
        }
    }
}

fn draw_hero_at(g: &G, hero: usize, hx: f32, feet: f32, t: f32) {
    let wave = (t * 5.0).sin() * 3.0;
    let skin = rgba(0.96, 0.8, 0.65, 1.0);
    match hero {
        0 => {
            let leg = rgba(0.30, 0.20, 0.12, 1.0);
            g.tri((hx - 5.0, feet - 30.0), (hx - 5.0, feet - 8.0), (hx - 20.0 + wave, feet - 6.0), rgba(0.15, 0.4, 0.2, 1.0));
            g.rect(hx - 6.0, feet - 10.0, 5.0, 10.0, leg);
            g.rect(hx + 1.0, feet - 10.0, 5.0, 10.0, leg);
            g.rect(hx - 7.0, feet - 3.0, 7.0, 3.0, rgba(0.15, 0.1, 0.06, 1.0));
            g.rect(hx, feet - 3.0, 7.0, 3.0, rgba(0.15, 0.1, 0.06, 1.0));
            g.rect(hx - 7.0, feet - 28.0, 14.0, 19.0, rgba(0.2, 0.5, 0.25, 1.0));
            g.rect(hx - 7.0, feet - 15.0, 14.0, 3.0, rgba(0.5, 0.32, 0.14, 1.0));
            g.rect(hx - 13.0, feet - 36.0, 5.0, 20.0, rgba(0.45, 0.28, 0.12, 1.0));
            for k in 0..3 {
                g.line(hx - 12.0 + k as f32 * 2.0, feet - 36.0, hx - 12.0 + k as f32 * 2.0, feet - 42.0, 1.2, rgba(0.9, 0.85, 0.7, 1.0));
            }
            g.rect(hx - 6.0, feet - 40.0, 12.0, 12.0, skin);
            g.tri((hx - 9.0, feet - 36.0), (hx, feet - 54.0), (hx + 9.0, feet - 36.0), rgba(0.18, 0.45, 0.22, 1.0));
            g.rect(hx - 9.0, feet - 42.0, 18.0, 6.0, rgba(0.18, 0.45, 0.22, 1.0));
            g.rect(hx + 1.0, feet - 36.0, 3.0, 3.0, BLACK);
        }
        1 => {
            let steel = rgba(0.68, 0.7, 0.8, 1.0);
            let steel_d = rgba(0.45, 0.47, 0.57, 1.0);
            g.tri((hx - 5.0, feet - 30.0), (hx - 5.0, feet - 6.0), (hx - 20.0 + wave, feet - 4.0), rgba(0.75, 0.1, 0.15, 1.0));
            g.rect(hx - 6.0, feet - 10.0, 5.0, 10.0, steel_d);
            g.rect(hx + 1.0, feet - 10.0, 5.0, 10.0, steel_d);
            g.rect(hx - 8.0, feet - 28.0, 16.0, 19.0, steel);
            g.rect(hx - 8.0, feet - 28.0, 16.0, 3.0, rgba(0.85, 0.87, 0.95, 1.0));
            g.rect(hx - 2.0, feet - 24.0, 4.0, 10.0, rgba(0.75, 0.12, 0.15, 1.0));
            g.rect(hx - 8.0, feet - 12.0, 16.0, 3.0, rgba(0.5, 0.35, 0.15, 1.0));
            g.circ(hx - 8.0, feet - 27.0, 5.0, steel);
            g.circ(hx + 8.0, feet - 27.0, 5.0, steel);
            g.rect(hx - 7.0, feet - 42.0, 14.0, 14.0, steel);
            g.rect(hx - 5.0, feet - 37.0, 11.0, 3.0, BLACK);
            g.tri((hx - 3.0, feet - 42.0), (hx + 2.0, feet - 56.0), (hx + 6.0, feet - 42.0), rgba(0.85, 0.12, 0.18, 1.0));
        }
        _ => {
            let robe = rgba(0.35, 0.18, 0.6, 1.0);
            g.tri((hx - 11.0, feet), (hx + 11.0, feet), (hx, feet - 32.0), robe);
            g.rect(hx - 7.0, feet - 32.0, 14.0, 20.0, robe);
            g.rect(hx - 7.0, feet - 18.0, 14.0, 3.0, rgba(1.0, 0.85, 0.3, 1.0));
            g.rect(hx - 6.0, feet - 42.0, 12.0, 11.0, skin);
            g.tri((hx - 6.0, feet - 33.0), (hx + 6.0, feet - 33.0), (hx + 1.0, feet - 22.0), rgba(0.95, 0.95, 0.98, 1.0));
            g.tri((hx - 12.0, feet - 40.0), (hx + 12.0, feet - 40.0), (hx + 3.0, feet - 66.0), rgba(0.3, 0.15, 0.55, 1.0));
            g.rect(hx - 12.0, feet - 42.0, 24.0, 4.0, rgba(0.25, 0.12, 0.45, 1.0));
            g.rect(hx - 6.0, feet - 46.0, 12.0, 3.0, rgba(1.0, 0.85, 0.3, 1.0));
            g.circ(hx + 1.0, feet - 54.0, 2.5, rgba(1.0, 0.9, 0.4, 1.0));
            g.rect(hx + 1.0, feet - 38.0, 3.0, 3.0, BLACK);
        }
    }
}

fn draw_weapon_at(g: &G, wi: usize, sh: Vec2, aim: Vec2, recoil: f32, ready: f32, t: f32) {
    let perp = vec2(-aim.y, aim.x);
    let grip = sh + aim * (13.0 - recoil * 4.0);
    let skin = rgba(0.96, 0.8, 0.65, 1.0);
    let wood = rgba(0.55, 0.34, 0.15, 1.0);
    let steel = rgba(0.85, 0.88, 0.95, 1.0);
    match wi {
        0 => {
            let bc = grip + aim * 4.0;
            let top = bc + perp * 20.0 - aim * 6.0;
            let bot = bc - perp * 20.0 - aim * 6.0;
            let mid = bc + aim * 6.0;
            g.line(top.x, top.y, mid.x, mid.y, 3.5, wood);
            g.line(mid.x, mid.y, bot.x, bot.y, 3.5, wood);
            let pull = 2.0 + 9.0 * ready;
            let sp = bc - aim * pull;
            let sc = rgba(1.0, 1.0, 1.0, 0.85);
            g.line(top.x, top.y, sp.x, sp.y, 1.2, sc);
            g.line(sp.x, sp.y, bot.x, bot.y, 1.2, sc);
            let tip = sp + aim * 26.0;
            g.line(sp.x, sp.y, tip.x, tip.y, 2.2, rgba(0.95, 0.85, 0.6, 1.0));
            let hd = tip + aim * 7.0;
            g.line(tip.x, tip.y, hd.x, hd.y, 3.5, steel);
        }
        1 => {
            let back = grip - aim * 14.0;
            let front = grip + aim * 18.0;
            g.line(back.x, back.y, front.x, front.y, 6.0, wood);
            g.line(back.x, back.y, front.x, front.y, 2.0, rgba(0.35, 0.2, 0.08, 1.0));
            let lt = front + perp * 20.0 - aim * 7.0;
            let lb = front - perp * 20.0 - aim * 7.0;
            g.line(lt.x, lt.y, front.x, front.y, 4.0, rgba(0.3, 0.3, 0.36, 1.0));
            g.line(front.x, front.y, lb.x, lb.y, 4.0, rgba(0.3, 0.3, 0.36, 1.0));
            let sp = front - aim * (3.0 + 6.0 * ready);
            g.line(lt.x, lt.y, sp.x, sp.y, 1.3, rgba(1.0, 1.0, 1.0, 0.8));
            g.line(sp.x, sp.y, lb.x, lb.y, 1.3, rgba(1.0, 1.0, 1.0, 0.8));
            let tip = sp + aim * 26.0;
            g.line(sp.x, sp.y, tip.x, tip.y, 3.0, rgba(0.8, 0.8, 0.88, 1.0));
            let hd = tip + aim * 8.0;
            g.line(tip.x, tip.y, hd.x, hd.y, 4.5, steel);
        }
        2 | 3 => {
            let a = grip - aim * 20.0;
            let b = grip + aim * 28.0;
            g.line(a.x, a.y, b.x, b.y, 4.0, rgba(0.4, 0.25, 0.12, 1.0));
            let orb = b + aim * 6.0;
            let pulse = 1.0 + 0.15 * (t * 8.0).sin();
            if wi == 2 {
                g.glow(orb.x, orb.y, 18.0 * pulse, rgba(1.0, 0.5, 0.1, 0.9));
                g.circ(orb.x, orb.y, 7.0 * pulse, rgba(1.0, 0.6, 0.15, 1.0));
                g.circ(orb.x, orb.y, 4.0, rgba(1.0, 0.95, 0.5, 1.0));
            } else {
                g.glow(orb.x, orb.y, 18.0 * pulse, rgba(0.4, 0.8, 1.0, 0.9));
                g.circ(orb.x, orb.y, 6.0 * pulse, rgba(0.5, 0.85, 1.0, 1.0));
                g.circ(orb.x, orb.y, 3.0, WHITE);
                for k in 0..3 {
                    let ang = t * 6.0 + k as f32 * 2.094;
                    g.line(orb.x, orb.y, orb.x + ang.cos() * 11.0, orb.y + ang.sin() * 11.0, 1.5, rgba(0.8, 0.95, 1.0, 0.9));
                }
            }
            g.ring(orb.x, orb.y, 9.0, 1.5, rgba(0.9, 0.8, 0.4, 1.0));
        }
        _ => {
            let a = grip - aim * 8.0;
            let b = grip + aim * 34.0;
            g.circ(sh.x, sh.y + 14.0, 12.0, rgba(0.4, 0.25, 0.1, 1.0));
            g.circ(sh.x, sh.y + 14.0, 4.0, rgba(0.25, 0.15, 0.06, 1.0));
            g.line(a.x, a.y, b.x, b.y, 15.0, rgba(0.22, 0.22, 0.27, 1.0));
            g.line(a.x, a.y, b.x, b.y, 6.0, rgba(0.42, 0.43, 0.5, 1.0));
            g.circ(b.x, b.y, 8.5, rgba(0.15, 0.15, 0.2, 1.0));
            let m = grip + aim * 14.0;
            g.circ(m.x, m.y, 8.0, rgba(0.3, 0.3, 0.36, 1.0));
        }
    }
    g.line(sh.x, sh.y, grip.x, grip.y, 4.0, skin);
}

fn cols(kd: &Kind, flash: bool) -> Pal {
    if flash {
        (WHITE, WHITE, WHITE)
    } else {
        (kd.color, shade(kd.color, 0.55), shade(kd.color, 1.35))
    }
}

fn eyes(g: &G, x1: f32, x2: f32, y: f32, r: f32, c: Color) {
    g.circ(x1, y, r, c);
    g.circ(x2, y, r, c);
    g.circ(x1 - r * 0.3, y, r * 0.5, rgba(0.9, 0.05, 0.05, 1.0));
    g.circ(x2 - r * 0.3, y, r * 0.5, rgba(0.9, 0.05, 0.05, 1.0));
}

fn draw_enemy(g: &G, e: &Enemy, t: f32) {
    let kd = kind(e.kind);
    let rc = enemy_rect(e, t);
    let cx = rc.x + rc.w * 0.5;
    let cy = rc.y + rc.h * 0.5;
    let pal = cols(&kd, e.hit > 0.0);
    let (col, dark, light) = pal;
    let ph = t * 9.0 + e.ph;
    let (x, y, w, h) = (rc.x, rc.y, rc.w, rc.h);
    let l1 = ph.sin().max(0.0) * 3.0;
    let l2 = (-ph.sin()).max(0.0) * 3.0;

    if e.elite {
        g.glow(cx, cy, w.max(h) * 0.8, rgba(1.0, 0.8, 0.2, 0.9));
    }
    if kd.boss {
        g.glow(cx, cy, h * 0.8, with_a(kd.color, 0.6));
    }
    if kd.flying {
        g.rect(cx - 12.0, GROUND + 6.0, 24.0, 4.0, rgba(0.0, 0.0, 0.0, 0.2));
    } else {
        g.rect(x - 2.0, y + h - 2.0, w + 4.0, 5.0, rgba(0.0, 0.0, 0.0, 0.3));
    }

    match e.kind {
        0 => {
            g.rect(x + w * 0.18, y + h - 9.0 - l1, 5.0, 9.0, dark);
            g.rect(x + w * 0.58, y + h - 9.0 - l2, 5.0, 9.0, dark);
            g.rect(x + 2.0, y + 14.0, w - 4.0, h - 24.0, col);
            g.rect(x + 2.0, y + h - 17.0, w - 4.0, 6.0, rgba(0.45, 0.3, 0.15, 1.0));
            g.circ(cx, y + 9.0, 9.0, col);
            g.tri((cx - 8.0, y + 7.0), (cx - 18.0, y + 3.0), (cx - 8.0, y + 13.0), col);
            g.tri((cx + 8.0, y + 7.0), (cx + 18.0, y + 3.0), (cx + 8.0, y + 13.0), col);
            eyes(g, cx - 3.5, cx + 3.5, y + 8.0, 2.6, rgba(1.0, 0.95, 0.3, 1.0));
            g.line(x - 1.0, y + 24.0, x - 10.0, y + 15.0, 2.0, rgba(0.88, 0.88, 0.95, 1.0));
        }
        1 => {
            let fl = (t * 16.0 + e.ph).sin() * 6.0;
            g.tri((x + w - 2.0, y + 10.0), (x + w + 15.0, y + fl), (x + w + 8.0, y + 22.0), dark);
            g.line(x + w - 2.0, y + h - 10.0, x + w + 8.0, y + h - 20.0 + fl * 0.4, 2.0, dark);
            g.tri((x + w + 6.0, y + h - 24.0), (x + w + 13.0, y + h - 22.0), (x + w + 8.0, y + h - 16.0), rgba(0.9, 0.15, 0.1, 1.0));
            g.rect(x + w * 0.2, y + h - 8.0 - l1, 4.0, 8.0, dark);
            g.rect(x + w * 0.58, y + h - 8.0 - l2, 4.0, 8.0, dark);
            g.rect(x + 2.0, y + 11.0, w - 4.0, h - 19.0, col);
            g.circ(cx, y + 9.0, 8.5, light);
            g.tri((cx - 7.0, y + 4.0), (cx - 6.0, y - 6.0), (cx - 2.0, y + 3.0), rgba(1.0, 0.85, 0.3, 1.0));
            g.tri((cx + 7.0, y + 4.0), (cx + 6.0, y - 6.0), (cx + 2.0, y + 3.0), rgba(1.0, 0.85, 0.3, 1.0));
            eyes(g, cx - 3.0, cx + 3.0, y + 9.0, 2.3, WHITE);
            g.glow(x - 3.0, y + 18.0, 8.0, rgba(1.0, 0.6, 0.1, 0.9));
            g.circ(x - 3.0, y + 18.0, 3.0, rgba(1.0, 0.8, 0.2, 1.0));
        }
        2 => {
            g.rect(x + w * 0.14, y + h - 11.0 - l1, 10.0, 11.0, dark);
            g.rect(x + w * 0.56, y + h - 11.0 - l2, 10.0, 11.0, dark);
            g.rect(x + 2.0, y + 16.0, w - 4.0, h - 28.0, col);
            g.circ(cx, y + h * 0.6, 12.0, light);
            g.circ(x + 4.0, y + 18.0, 8.0, rgba(0.4, 0.4, 0.45, 1.0));
            g.circ(x + w - 4.0, y + 18.0, 8.0, rgba(0.4, 0.4, 0.45, 1.0));
            g.circ(cx, y + 11.0, 11.0, dark);
            g.tri((cx - 6.0, y + 17.0), (cx - 4.0, y + 9.0), (cx - 2.0, y + 17.0), WHITE);
            g.tri((cx + 6.0, y + 17.0), (cx + 4.0, y + 9.0), (cx + 2.0, y + 17.0), WHITE);
            g.line(cx - 8.0, y + 5.0, cx - 2.0, y + 8.0, 2.0, BLACK);
            g.line(cx + 8.0, y + 5.0, cx + 2.0, y + 8.0, 2.0, BLACK);
            eyes(g, cx - 4.5, cx + 4.5, y + 10.0, 2.3, rgba(1.0, 0.9, 0.3, 1.0));
            g.line(x - 2.0, y + h * 0.65, x - 14.0, y + 8.0, 5.0, rgba(0.45, 0.28, 0.12, 1.0));
            g.circ(x - 15.0, y + 6.0, 6.0, rgba(0.4, 0.25, 0.1, 1.0));
        }
        3 => {
            let flap = (t * 14.0 + e.ph).sin() * 10.0;
            g.tri((cx - 3.0, cy - 2.0), (cx - 26.0, cy - 10.0 + flap), (cx - 18.0, cy + 8.0), dark);
            g.tri((cx + 3.0, cy - 2.0), (cx + 26.0, cy - 10.0 + flap), (cx + 18.0, cy + 8.0), dark);
            g.circ(cx, cy, 8.0, col);
            g.tri((cx - 6.0, cy - 5.0), (cx - 4.0, cy - 13.0), (cx - 1.0, cy - 6.0), dark);
            g.tri((cx + 6.0, cy - 5.0), (cx + 4.0, cy - 13.0), (cx + 1.0, cy - 6.0), dark);
            eyes(g, cx - 3.0, cx + 3.0, cy - 1.0, 1.8, rgba(1.0, 0.3, 0.2, 1.0));
            g.tri((cx - 2.0, cy + 3.0), (cx - 1.0, cy + 7.0), (cx, cy + 3.0), WHITE);
        }
        4 => {
            let bone = col;
            g.rect(x + w * 0.25, y + h - 12.0 - l1, 3.0, 12.0, bone);
            g.rect(x + w * 0.6, y + h - 12.0 - l2, 3.0, 12.0, bone);
            g.rect(x + 5.0, y + 17.0, w - 10.0, 14.0, bone);
            for k in 0..3 {
                g.rect(x + 5.0, y + 20.0 + k as f32 * 4.0, w - 10.0, 1.5, dark);
            }
            g.rect(cx - 1.0, y + 16.0, 2.0, 15.0, dark);
            g.circ(cx, y + 9.0, 8.5, light);
            g.rect(cx - 5.0, y + 14.0, 10.0, 4.0, light);
            g.circ(cx - 3.5, y + 8.0, 2.5, BLACK);
            g.circ(cx + 3.5, y + 8.0, 2.5, BLACK);
            g.circ(cx - 3.5, y + 8.0, 1.0, rgba(1.0, 0.2, 0.1, 1.0));
            g.circ(cx + 3.5, y + 8.0, 1.0, rgba(1.0, 0.2, 0.1, 1.0));
            g.line(x - 2.0, y + 14.0, x - 8.0, y + 24.0, 2.5, rgba(0.5, 0.32, 0.15, 1.0));
            g.line(x - 8.0, y + 24.0, x - 2.0, y + 34.0, 2.5, rgba(0.5, 0.32, 0.15, 1.0));
            g.line(x - 2.0, y + 14.0, x - 2.0, y + 34.0, 1.0, rgba(1.0, 1.0, 1.0, 0.7));
        }
        5 | 11 => {
            let sq = 1.0 + 0.1 * (ph * 0.7).sin();
            let r = w * 0.5 * sq;
            g.rect(x, y + h * 0.55, w, h * 0.45, with_a(col, 0.92));
            g.circ(cx, y + h * 0.55, r, with_a(col, 0.92));
            g.circ(cx - r * 0.35, y + h * 0.35, r * 0.22, rgba(1.0, 1.0, 1.0, 0.55));
            g.circ(cx - w * 0.2, y + h * 0.55, w * 0.1 + 1.0, WHITE);
            g.circ(cx + w * 0.2, y + h * 0.55, w * 0.1 + 1.0, WHITE);
            g.circ(cx - w * 0.22, y + h * 0.56, w * 0.05 + 0.5, BLACK);
            g.circ(cx + w * 0.18, y + h * 0.56, w * 0.05 + 0.5, BLACK);
        }
        6 => {
            let run = (ph * 1.4).sin() * 5.0;
            g.line(x + 9.0, y + 16.0, x + 4.0 + run, y + h, 3.0, dark);
            g.line(x + 14.0, y + 16.0, x + 16.0 - run, y + h, 3.0, dark);
            g.line(x + w - 10.0, y + 16.0, x + w - 6.0 + run, y + h, 3.0, dark);
            g.line(x + w - 15.0, y + 16.0, x + w - 18.0 - run, y + h, 3.0, dark);
            g.rect(x + 7.0, y + 4.0, w - 12.0, 14.0, col);
            g.rect(x + 7.0, y + 4.0, w - 12.0, 4.0, light);
            g.tri((x + w - 6.0, y + 6.0), (x + w + 10.0, y - 2.0 + run * 0.4), (x + w - 6.0, y + 12.0), col);
            g.tri((x + 12.0, y + 6.0), (x - 6.0, y + 12.0), (x + 12.0, y + 17.0), col);
            g.tri((x + 10.0, y + 4.0), (x + 14.0, y - 5.0), (x + 18.0, y + 4.0), dark);
            g.circ(x + 12.0, y + 8.0, 2.0, rgba(1.0, 0.2, 0.1, 1.0));
            g.tri((x - 4.0, y + 13.0), (x - 2.0, y + 18.0), (x, y + 13.0), WHITE);
        }
        7 => {
            let steel = col;
            g.rect(x + w * 0.2, y + h - 11.0 - l1, 6.0, 11.0, dark);
            g.rect(x + w * 0.55, y + h - 11.0 - l2, 6.0, 11.0, dark);
            g.rect(x + 3.0, y + 16.0, w - 6.0, h - 28.0, steel);
            g.rect(x + 3.0, y + 16.0, w - 6.0, 3.0, light);
            g.rect(cx - 2.0, y + 22.0, 4.0, 10.0, rgba(0.7, 0.15, 0.15, 1.0));
            g.rect(cx - 8.0, y + 4.0, 16.0, 15.0, steel);
            g.rect(cx - 8.0, y + 10.0, 14.0, 3.0, BLACK);
            g.tri((cx - 3.0, y + 4.0), (cx + 4.0, y - 8.0), (cx + 8.0, y + 4.0), rgba(0.2, 0.4, 0.9, 1.0));
            g.circ(x - 1.0, y + 28.0, 12.0, rgba(0.2, 0.35, 0.8, 1.0));
            g.ring(x - 1.0, y + 28.0, 12.0, 2.0, rgba(0.85, 0.85, 0.95, 1.0));
            g.line(x - 1.0, y + 20.0, x - 1.0, y + 36.0, 2.0, rgba(1.0, 0.85, 0.3, 1.0));
            g.line(x + w + 2.0, y + 18.0, x + w + 10.0, y + 2.0, 2.5, rgba(0.9, 0.92, 1.0, 1.0));
        }
        8 => {
            let flick = ((t * 14.0).sin() * 0.5 + 0.5) * 0.7 + 0.3;
            g.rect(x + w * 0.2, y + h - 6.0 - l1, 5.0, 6.0, dark);
            g.rect(x + w * 0.58, y + h - 6.0 - l2, 5.0, 6.0, dark);
            g.circ(cx, y + h * 0.58, 12.5, rgba(0.15, 0.1, 0.1, 1.0));
            g.circ(cx - 4.0, y + h * 0.5, 4.0, rgba(1.0, 1.0, 1.0, 0.15));
            g.rect(cx - 12.0, y + h * 0.58 - 2.0, 24.0, 4.0, col);
            g.line(cx + 2.0, y + 4.0, cx + 8.0, y - 5.0, 2.0, rgba(0.6, 0.45, 0.2, 1.0));
            g.glow(cx + 8.0, y - 6.0, 10.0, rgba(1.0, 0.8, 0.2, flick));
            g.circ(cx + 8.0, y - 6.0, 3.0, rgba(1.0, 0.95, 0.5, 1.0));
            eyes(g, cx - 4.0, cx + 4.0, y + h * 0.55, 2.4, WHITE);
        }
        9 => {
            let wv = (t * 3.0 + e.ph).sin() * 4.0;
            g.glow(cx, cy, 26.0, rgba(0.4, 0.7, 1.0, 0.5));
            g.tri((cx - 15.0, cy + 18.0), (cx + 15.0, cy + 18.0), (cx, cy - 8.0), with_a(col, 0.85));
            for k in 0..3 {
                let tx = cx - 11.0 + k as f32 * 11.0;
                g.tri((tx - 5.0, cy + 18.0), (tx + 5.0, cy + 18.0), (tx + wv * (k as f32 - 1.0), cy + 28.0), with_a(dark, 0.7));
            }
            g.circ(cx, cy - 8.0, 10.0, with_a(dark, 0.95));
            g.circ(cx, cy - 6.0, 6.5, rgba(0.02, 0.02, 0.08, 1.0));
            g.circ(cx - 3.0, cy - 7.0, 1.8, rgba(0.5, 1.0, 1.0, 1.0));
            g.circ(cx + 3.0, cy - 7.0, 1.8, rgba(0.5, 1.0, 1.0, 1.0));
        }
        10 => {
            let bob = (t * 4.0 + e.ph).sin() * 2.0;
            g.tri((x + 2.0, y + h), (x + w - 2.0, y + h), (cx, y + 14.0), col);
            g.tri((cx, y + 14.0), (x + w - 2.0, y + h), (cx + 6.0, y + h), dark);
            g.circ(cx, y + 11.0, 8.0, rgba(0.9, 0.88, 0.75, 1.0));
            g.tri((cx - 8.0, y + 6.0), (cx, y - 6.0), (cx + 8.0, y + 6.0), rgba(0.9, 0.3, 0.2, 1.0));
            g.rect(cx - 4.0, y + 9.0, 3.0, 3.0, BLACK);
            g.rect(cx + 1.0, y + 9.0, 3.0, 3.0, BLACK);
            g.line(x - 3.0, y + h - 2.0, x - 3.0, y + 4.0, 2.5, rgba(0.45, 0.3, 0.15, 1.0));
            g.glow(x - 3.0, y + 2.0 + bob, 11.0, rgba(0.3, 1.0, 0.5, 0.9));
            g.circ(x - 3.0, y + 2.0 + bob, 4.0, rgba(0.6, 1.0, 0.7, 1.0));
            for k in 0..2 {
                let a = t * 3.0 + k as f32 * 3.14;
                g.circ(cx + a.cos() * 22.0, cy + a.sin() * 8.0, 2.5, rgba(0.4, 1.0, 0.6, 0.9));
            }
        }
        12 => {
            g.rect(x + 12.0, y + h - 24.0 - l1, 26.0, 24.0, dark);
            g.rect(x + w - 38.0, y + h - 24.0 - l2, 26.0, 24.0, dark);
            g.rect(x + 6.0, y + 34.0, w - 12.0, h - 52.0, col);
            g.circ(cx, y + h * 0.58, 26.0, light);
            g.circ(x + 8.0, y + 40.0, 16.0, rgba(0.4, 0.4, 0.45, 1.0));
            g.circ(x + w - 8.0, y + 40.0, 16.0, rgba(0.4, 0.4, 0.45, 1.0));
            g.circ(cx - 4.0, y + 24.0, 22.0, dark);
            g.rect(cx - 20.0, y + 2.0, 32.0, 8.0, rgba(1.0, 0.82, 0.2, 1.0));
            for k in 0..4 {
                let sx = cx - 20.0 + k as f32 * 9.0;
                g.tri((sx, y + 2.0), (sx + 4.5, y - 12.0), (sx + 9.0, y + 2.0), rgba(1.0, 0.82, 0.2, 1.0));
            }
            g.tri((cx - 14.0, y + 38.0), (cx - 10.0, y + 22.0), (cx - 5.0, y + 38.0), WHITE);
            g.tri((cx + 6.0, y + 38.0), (cx + 10.0, y + 22.0), (cx + 15.0, y + 38.0), WHITE);
            eyes(g, cx - 12.0, cx + 4.0, y + 22.0, 4.5, rgba(1.0, 0.9, 0.3, 1.0));
            g.line(x - 6.0, y + 58.0, x - 38.0, y + 10.0, 14.0, rgba(0.45, 0.28, 0.12, 1.0));
            g.circ(x - 40.0, y + 6.0, 17.0, rgba(0.4, 0.25, 0.1, 1.0));
            for k in 0..5 {
                let a = k as f32 * 1.2;
                g.circ(x - 40.0 + a.cos() * 14.0, y + 6.0 + a.sin() * 14.0, 3.0, rgba(0.8, 0.8, 0.85, 1.0));
            }
        }
        13 => {
            let fl = (t * 5.0 + e.ph).sin() * 26.0;
            g.tri((cx + 10.0, cy - 8.0), (cx + 70.0, cy - 58.0 + fl), (cx + 54.0, cy + 8.0), dark);
            g.tri((cx + 10.0, cy - 8.0), (cx + 40.0, cy - 40.0 + fl), (cx + 70.0, cy - 58.0 + fl), light);
            g.tri((cx + 40.0, cy + 4.0), (cx + 84.0, cy + 20.0), (cx + 52.0, cy + 14.0), dark);
            g.circ(cx + 10.0, cy, 28.0, col);
            g.circ(cx + 8.0, cy + 8.0, 18.0, rgba(0.95, 0.75, 0.3, 1.0));
            g.circ(cx - 34.0, cy - 12.0, 17.0, col);
            g.tri((cx - 46.0, cy - 8.0), (cx - 72.0, cy - 2.0), (cx - 44.0, cy + 6.0), col);
            g.tri((cx - 34.0, cy - 26.0), (cx - 24.0, cy - 46.0), (cx - 24.0, cy - 24.0), rgba(0.95, 0.9, 0.8, 1.0));
            g.circ(cx - 38.0, cy - 16.0, 4.0, rgba(1.0, 0.95, 0.2, 1.0));
            g.circ(cx - 38.0, cy - 16.0, 1.6, BLACK);
            let fa = 0.6 + 0.4 * (t * 8.0).sin();
            g.glow(cx - 66.0, cy, 14.0, rgba(1.0, 0.5, 0.1, fa));
            g.circ(cx - 62.0, cy, 4.0, rgba(1.0, 0.8, 0.3, fa));
        }
        _ => {
            let hov = 8.0 + (t * 2.0 + e.ph).sin() * 4.0;
            let yy = y - hov;
            g.tri((x + 6.0, yy + h), (x + w - 6.0, yy + h), (cx, yy + 22.0), col);
            g.tri((x - 4.0, yy + h), (x + 12.0, yy + h), (cx, yy + 26.0), dark);
            for k in 0..4 {
                let tx = x + 8.0 + k as f32 * 14.0;
                g.tri((tx - 5.0, yy + h - 4.0), (tx + 5.0, yy + h - 4.0), (tx, yy + h + 8.0 + (t * 4.0 + k as f32).sin() * 3.0), with_a(dark, 0.8));
            }
            g.circ(cx, yy + 16.0, 12.0, rgba(0.92, 0.9, 0.82, 1.0));
            g.rect(cx - 8.0, yy + 22.0, 16.0, 6.0, rgba(0.92, 0.9, 0.82, 1.0));
            g.circ(cx - 4.5, yy + 15.0, 3.2, BLACK);
            g.circ(cx + 4.5, yy + 15.0, 3.2, BLACK);
            g.glow(cx - 4.5, yy + 15.0, 7.0, rgba(0.3, 1.0, 0.4, 0.9));
            g.glow(cx + 4.5, yy + 15.0, 7.0, rgba(0.3, 1.0, 0.4, 0.9));
            g.rect(cx - 12.0, yy + 3.0, 24.0, 4.0, rgba(0.55, 0.25, 0.8, 1.0));
            for k in 0..3 {
                let sx = cx - 12.0 + k as f32 * 8.0;
                g.tri((sx, yy + 3.0), (sx + 4.0, yy - 8.0), (sx + 8.0, yy + 3.0), rgba(0.55, 0.25, 0.8, 1.0));
            }
            g.line(x - 8.0, yy + h, x - 8.0, yy + 8.0, 3.0, rgba(0.4, 0.28, 0.14, 1.0));
            g.glow(x - 8.0, yy + 4.0, 14.0, rgba(0.6, 0.3, 1.0, 0.9));
            g.circ(x - 8.0, yy + 4.0, 5.0, rgba(0.9, 0.85, 0.8, 1.0));
            for k in 0..3 {
                let a = t * 2.0 + k as f32 * 2.094;
                let ox = cx + a.cos() * 40.0;
                let oy = yy + 40.0 + a.sin() * 12.0;
                g.glow(ox, oy, 9.0, rgba(0.7, 0.3, 1.0, 0.9));
                g.circ(ox, oy, 3.0, rgba(0.9, 0.7, 1.0, 1.0));
            }
        }
    }

    if e.slow > 0.0 {
        g.rect(x, y, w, h, rgba(0.55, 0.85, 1.0, 0.35));
        g.tri((cx - 6.0, y), (cx, y - 9.0), (cx + 6.0, y), rgba(0.8, 0.95, 1.0, 0.85));
    }
    if e.hp < e.max_hp || kd.boss {
        let bh = if kd.boss { 7.0 } else { 4.0 };
        let by = y - if kd.boss { 22.0 } else { 9.0 };
        g.rect(x, by, w, bh, rgba(0.35, 0.04, 0.04, 0.9));
        let f = (e.hp / e.max_hp).clamp(0.0, 1.0);
        let bc = if e.elite { rgba(1.0, 0.8, 0.2, 1.0) } else { rgba(0.2, 0.9, 0.3, 1.0) };
        g.rect(x, by, w * f, bh, bc);
    }
}

// =====================================================================
//  drawing: ui
// =====================================================================

fn draw_icon(g: &G, i: usize, cx: f32, cy: f32) {
    match i {
        0 => {
            g.line(cx + 12.0, cy - 14.0, cx - 2.0, cy + 2.0, 6.0, rgba(1.0, 0.5, 0.1, 0.5));
            g.circ(cx - 2.0, cy + 3.0, 9.0, rgba(1.0, 0.5, 0.1, 1.0));
            g.circ(cx - 2.0, cy + 3.0, 5.0, rgba(1.0, 0.9, 0.4, 1.0));
        }
        1 => {
            for k in 0..3 {
                let a = k as f32 * 1.0472;
                let (s, c) = a.sin_cos();
                g.line(cx - c * 13.0, cy - s * 13.0, cx + c * 13.0, cy + s * 13.0, 2.5, rgba(0.7, 0.92, 1.0, 1.0));
            }
            g.circ(cx, cy, 3.0, WHITE);
        }
        _ => {
            g.tri((cx - 9.0, cy + 12.0), (cx + 9.0, cy + 12.0), (cx, cy - 15.0), rgba(0.95, 0.3, 0.1, 1.0));
            g.tri((cx - 5.0, cy + 12.0), (cx + 5.0, cy + 12.0), (cx, cy - 3.0), rgba(1.0, 0.85, 0.3, 1.0));
        }
    }
}

fn draw_coin(g: &G, x: f32, y: f32, r: f32) {
    g.circ(x, y, r, rgba(1.0, 0.82, 0.2, 1.0));
    g.circ(x, y, r * 0.62, rgba(0.9, 0.62, 0.1, 1.0));
    g.circ(x - r * 0.25, y - r * 0.25, r * 0.2, rgba(1.0, 0.95, 0.7, 1.0));
}

impl Game {
    fn draw_shop(&self, g: &G) {
        let p = shop_panel(self.lw);
        g.rect(0.0, 0.0, self.lw, BASE_H, rgba(0.0, 0.0, 0.0, 0.65));
        g.rect(p.x, p.y, p.w, p.h, rgba(0.06, 0.07, 0.15, 1.0));
        g.frame(p.x, p.y, p.w, p.h, 3.0, rgba(0.45, 0.5, 0.9, 1.0));
        let names = ["UPGRADES", "WEAPONS", "HEROES"];
        for i in 0..3 {
            let r = tab_rect(p, i);
            let on = self.tab == i;
            g.rect(r.x, r.y, r.w, r.h, if on { rgba(0.25, 0.3, 0.7, 1.0) } else { rgba(0.12, 0.13, 0.26, 1.0) });
            g.frame(r.x, r.y, r.w, r.h, 2.0, if on { rgba(0.7, 0.8, 1.0, 1.0) } else { rgba(0.3, 0.32, 0.55, 1.0) });
            g.text_center(names[i], r.x + r.w / 2.0, r.y + 30.0, 24.0, WHITE);
        }
        let c = close_rect(p);
        g.rect(c.x, c.y, c.w, c.h, rgba(0.6, 0.15, 0.2, 1.0));
        g.frame(c.x, c.y, c.w, c.h, 2.0, rgba(1.0, 0.5, 0.5, 1.0));
        g.text_center("CLOSE", c.x + c.w / 2.0, c.y + 30.0, 24.0, WHITE);

        draw_coin(g, p.x + 26.0, p.y + 76.0, 10.0);
        g.text(&fmt(self.gold as f64), p.x + 44.0, p.y + 85.0, 30.0, rgba(1.0, 0.9, 0.4, 1.0));
        if self.msg_t > 0.0 {
            g.text_center(&self.msg, p.x + p.w * 0.62, p.y + 84.0, 24.0, rgba(1.0, 1.0, 0.6, 1.0));
        }

        let (n, cols, rows, mh) = tab_grid(self.tab);
        for i in 0..n {
            let rc = card_rect(p, cols, rows, i, mh);
            match self.tab {
                0 => self.draw_up_card(g, rc, i),
                1 => self.draw_weapon_card(g, rc, i),
                _ => self.draw_hero_card(g, rc, i),
            }
        }
    }

    fn draw_up_card(&self, g: &G, rc: Rect, i: usize) {
        let lvl = self.up[i];
        let maxed = lvl >= UP_MAX[i];
        let cost = up_cost(i, lvl);
        let can = !maxed && self.gold >= cost;
        g.rect(rc.x, rc.y, rc.w, rc.h, if can { rgba(0.13, 0.27, 0.2, 1.0) } else { rgba(0.11, 0.12, 0.22, 1.0) });
        g.frame(rc.x, rc.y, rc.w, rc.h, 2.0, if can { rgba(0.3, 0.9, 0.45, 1.0) } else { rgba(0.3, 0.32, 0.5, 1.0) });
        g.text(UP_NAME[i], rc.x + 12.0, rc.y + 24.0, 24.0, WHITE);
        let lv = if maxed { "MAX".to_string() } else { format!("Lv {}", lvl) };
        g.text_right(&lv, rc.x + rc.w - 12.0, rc.y + 24.0, 22.0, rgba(0.7, 0.85, 1.0, 1.0));
        g.text(UP_DESC[i], rc.x + 12.0, rc.y + 45.0, 17.0, rgba(0.75, 0.77, 0.9, 1.0));
        g.text(&format!("Now: {}", self.up_now(i)), rc.x + 12.0, rc.y + rc.h - 8.0, 18.0, rgba(0.6, 0.9, 0.6, 1.0));
        if !maxed {
            let cc = if can { rgba(1.0, 0.9, 0.4, 1.0) } else { rgba(0.6, 0.6, 0.65, 1.0) };
            g.text_right(&fmt(cost as f64), rc.x + rc.w - 12.0, rc.y + rc.h - 8.0, 22.0, cc);
            draw_coin(g, rc.x + rc.w - 18.0 - g.text_w(&fmt(cost as f64), 22.0) - 8.0, rc.y + rc.h - 14.0, 7.0);
        }
    }

    fn draw_weapon_card(&self, g: &G, rc: Rect, i: usize) {
        let w = weapon(i);
        let owned = self.owned_w[i];
        let eq = self.weapon == i;
        let can = !owned && self.gold >= w.price;
        let bg = if eq { rgba(0.18, 0.28, 0.5, 1.0) } else if owned || can { rgba(0.13, 0.27, 0.2, 1.0) } else { rgba(0.11, 0.12, 0.22, 1.0) };
        g.rect(rc.x, rc.y, rc.w, rc.h, bg);
        g.frame(rc.x, rc.y, rc.w, rc.h, 2.0, if eq { rgba(0.6, 0.8, 1.0, 1.0) } else if owned || can { rgba(0.3, 0.9, 0.45, 1.0) } else { rgba(0.3, 0.32, 0.5, 1.0) });
        let s = 1.8;
        let gw = G { u: g.u * s, ox: (rc.x + 46.0) / s, oy: (rc.y + rc.h * 0.55) / s };
        draw_weapon_at(&gw, i, vec2(0.0, 0.0), vec2(0.95, -0.25).normalize(), 0.0, 1.0, self.t);
        let tx = rc.x + 150.0;
        g.text(w.name, tx, rc.y + 30.0, 26.0, WHITE);
        g.text(w.desc, tx, rc.y + 54.0, 18.0, rgba(0.75, 0.77, 0.9, 1.0));
        g.text(&format!("DMG {}   RATE {:.1}/s", fmt(w.dmg as f64), 1.0 / w.cd), tx, rc.y + 78.0, 18.0, rgba(0.6, 0.9, 0.6, 1.0));
        if eq {
            g.text("EQUIPPED", tx, rc.y + rc.h - 14.0, 24.0, rgba(0.6, 0.85, 1.0, 1.0));
        } else if owned {
            g.text("OWNED - tap to equip", tx, rc.y + rc.h - 14.0, 22.0, rgba(0.5, 1.0, 0.6, 1.0));
        } else {
            let cc = if can { rgba(1.0, 0.9, 0.4, 1.0) } else { rgba(0.6, 0.6, 0.65, 1.0) };
            draw_coin(g, tx + 8.0, rc.y + rc.h - 21.0, 8.0);
            g.text(&fmt(w.price as f64), tx + 22.0, rc.y + rc.h - 14.0, 26.0, cc);
        }
    }

    fn draw_hero_card(&self, g: &G, rc: Rect, i: usize) {
        let (name, desc, price) = hero_info(i);
        let owned = self.owned_h[i];
        let eq = self.hero == i;
        let can = !owned && self.gold >= price;
        let bg = if eq { rgba(0.18, 0.28, 0.5, 1.0) } else if owned || can { rgba(0.13, 0.27, 0.2, 1.0) } else { rgba(0.11, 0.12, 0.22, 1.0) };
        g.rect(rc.x, rc.y, rc.w, rc.h, bg);
        g.frame(rc.x, rc.y, rc.w, rc.h, 2.0, if eq { rgba(0.6, 0.8, 1.0, 1.0) } else if owned || can { rgba(0.3, 0.9, 0.45, 1.0) } else { rgba(0.3, 0.32, 0.5, 1.0) });
        let s = 3.0;
        let gh = G { u: g.u * s, ox: (rc.x + rc.w * 0.5 - 10.0) / s, oy: (rc.y + 190.0) / s };
        draw_hero_at(&gh, i, 0.0, 0.0, self.t);
        draw_weapon_at(&gh, self.weapon, vec2(2.0, -24.0), vec2(0.95, -0.2).normalize(), 0.0, 1.0, self.t);
        g.text_center(name, rc.x + rc.w * 0.5, rc.y + 232.0, 28.0, WHITE);
        g.text_center(desc, rc.x + rc.w * 0.5, rc.y + 258.0, 18.0, rgba(0.75, 0.77, 0.9, 1.0));
        let by = rc.y + rc.h - 22.0;
        if eq {
            g.text_center("SELECTED", rc.x + rc.w * 0.5, by, 24.0, rgba(0.6, 0.85, 1.0, 1.0));
        } else if owned {
            g.text_center("OWNED - tap to select", rc.x + rc.w * 0.5, by, 22.0, rgba(0.5, 1.0, 0.6, 1.0));
        } else {
            let cc = if can { rgba(1.0, 0.9, 0.4, 1.0) } else { rgba(0.6, 0.6, 0.65, 1.0) };
            let s = fmt(price as f64);
            let tw = g.text_w(&s, 26.0);
            draw_coin(g, rc.x + rc.w * 0.5 - tw * 0.5 - 12.0, by - 7.0, 8.0);
            g.text_center(&s, rc.x + rc.w * 0.5 + 8.0, by, 26.0, cc);
        }
    }

    fn draw_buttons(&self, g: &G) {
        g.rect(0.0, UI_TOP, self.lw, BASE_H - UI_TOP, rgba(0.06, 0.07, 0.14, 0.96));
        g.rect(0.0, UI_TOP, self.lw, 2.0, rgba(0.35, 0.38, 0.7, 1.0));

        let s = btn_shop();
        g.rect(s.x, s.y, s.w, s.h, rgba(0.35, 0.25, 0.08, 1.0));
        g.frame(s.x, s.y, s.w, s.h, 2.0, rgba(1.0, 0.8, 0.3, 1.0));
        draw_coin(g, s.x + 26.0, s.y + 28.0, 11.0);
        g.text("SHOP", s.x + 46.0, s.y + 36.0, 28.0, WHITE);

        let a = btn_auto();
        let on = self.auto;
        g.rect(a.x, a.y, a.w, a.h, if on { rgba(0.14, 0.3, 0.2, 1.0) } else { rgba(0.22, 0.12, 0.14, 1.0) });
        g.frame(a.x, a.y, a.w, a.h, 2.0, if on { rgba(0.3, 0.9, 0.45, 1.0) } else { rgba(0.8, 0.35, 0.35, 1.0) });
        g.text_center("AUTO", a.x + a.w / 2.0, a.y + 26.0, 24.0, WHITE);
        g.text_center(if on { "ON" } else { "OFF" }, a.x + a.w / 2.0, a.y + 48.0, 22.0, if on { rgba(0.5, 1.0, 0.6, 1.0) } else { rgba(1.0, 0.5, 0.5, 1.0) });

        let names = ["Meteor", "Frost", "Rage"];
        for i in 0..3 {
            let r = btn_skill(self.lw, i);
            let cd = self.sk_cd[i];
            let ready = cd <= 0.0;
            g.rect(r.x, r.y, r.w, r.h, if ready { rgba(0.16, 0.28, 0.2, 1.0) } else { rgba(0.12, 0.13, 0.24, 1.0) });
            if !ready {
                let f = (cd / self.skill_cd(i)).clamp(0.0, 1.0);
                g.rect(r.x, r.y + r.h * (1.0 - f), r.w, r.h * f, rgba(0.0, 0.0, 0.0, 0.45));
            }
            let bc = if ready { rgba(1.0, 0.8, 0.3, 1.0) } else { rgba(0.4, 0.42, 0.55, 1.0) };
            g.frame(r.x, r.y, r.w, r.h, 2.0, bc);
            draw_icon(g, i, r.x + 24.0, r.y + 28.0);
            g.text(names[i], r.x + 46.0, r.y + 24.0, 22.0, WHITE);
            if ready {
                g.text("READY", r.x + 46.0, r.y + 46.0, 20.0, rgba(1.0, 0.9, 0.4, 1.0));
            } else {
                g.text(&format!("{:.0}s", cd.ceil()), r.x + 46.0, r.y + 46.0, 22.0, GRAY);
            }
        }
    }

    fn draw_hud(&self, g: &G, b: &Biome) {
        // left panel
        g.rect(8.0, 8.0, 190.0, if self.combo >= 3 { 120.0 } else { 98.0 }, rgba(0.0, 0.0, 0.0, 0.45));
        draw_coin(g, 26.0, 28.0, 10.0);
        g.text(&fmt(self.gold as f64), 44.0, 37.0, 30.0, rgba(1.0, 0.9, 0.4, 1.0));
        g.text(&format!("Wave {}", self.wave), 16.0, 62.0, 26.0, WHITE);
        g.text(&format!("Kills {}", self.kills), 16.0, 84.0, 22.0, LIGHTGRAY);
        if self.combo >= 3 {
            g.text(&format!("COMBO x{}", self.combo), 16.0, 110.0, 24.0, rgba(1.0, 0.6, 0.2, 1.0));
        }

        // castle hp bar
        let bw = (self.lw * 0.32).clamp(220.0, 420.0);
        let bx = (self.lw - bw) / 2.0;
        let f = (self.hp / self.hp_max()).clamp(0.0, 1.0);
        g.rect(bx - 3.0, 7.0, bw + 6.0, 28.0, rgba(0.0, 0.0, 0.0, 0.55));
        g.rect(bx, 10.0, bw, 22.0, rgba(0.35, 0.05, 0.05, 1.0));
        let hc = if f > 0.5 { rgba(0.2, 0.85, 0.3, 1.0) } else if f > 0.25 { rgba(0.95, 0.7, 0.15, 1.0) } else { rgba(0.95, 0.2, 0.2, 1.0) };
        g.rect(bx, 10.0, bw * f, 22.0, hc);
        g.rect(bx, 10.0, bw * f, 6.0, rgba(1.0, 1.0, 1.0, 0.2));
        g.text_center(&format!("CASTLE {} / {}", fmt(self.hp as f64), fmt(self.hp_max() as f64)), bx + bw / 2.0, 28.0, 20.0, WHITE);

        // boss bar
        for e in self.enemies.iter() {
            if kind(e.kind).boss {
                let bf = (e.hp / e.max_hp).clamp(0.0, 1.0);
                g.rect(bx - 3.0, 40.0, bw + 6.0, 24.0, rgba(0.0, 0.0, 0.0, 0.55));
                g.rect(bx, 43.0, bw, 18.0, rgba(0.25, 0.05, 0.3, 1.0));
                g.rect(bx, 43.0, bw * bf, 18.0, rgba(0.75, 0.2, 0.9, 1.0));
                g.text_center("BOSS", bx + bw / 2.0, 58.0, 18.0, WHITE);
                break;
            }
        }

        // right info
        g.text_right(b.name, self.lw - 14.0, 28.0, 22.0, with_a(WHITE, 0.85));
        g.text_right(&format!("Best wave {}", self.best), self.lw - 14.0, 50.0, 20.0, LIGHTGRAY);
        if self.rage > 0.0 {
            g.text_right(&format!("RAGE {:.0}s", self.rage.ceil()), self.lw - 14.0, 74.0, 24.0, rgba(1.0, 0.45, 0.2, 1.0));
        }
        if self.msg_t > 0.0 && !self.shop {
            g.text_center(&self.msg, self.lw / 2.0, 100.0, 24.0, rgba(0.7, 1.0, 0.7, self.msg_t.min(1.0)));
        }

        // wave banner
        if self.banner_t > 0.0 {
            let a = self.banner_t.min(1.0);
            let txt = format!("WAVE {}", self.wave);
            g.text_center(&txt, self.lw / 2.0 + 3.0, 203.0, 76.0, rgba(0.0, 0.0, 0.0, a * 0.6));
            g.text_center(&txt, self.lw / 2.0, 200.0, 76.0, rgba(1.0, 0.9, 0.5, a));
            if (self.wave - 1) % 10 == 0 {
                g.text_center(b.name, self.lw / 2.0, 236.0, 30.0, rgba(0.8, 0.9, 1.0, a));
            }
            if self.wave % 10 == 0 {
                g.text_center("BOSS WAVE!", self.lw / 2.0, 270.0, 36.0, rgba(1.0, 0.3, 0.3, a));
            }
        }
    }
}

impl Game {
    fn draw(&self, u: f32) {
        let g0 = G { u, ox: 0.0, oy: 0.0 };
        let s = self.shake * 8.0;
        let gs = G { u, ox: rnd(-1.0, 1.0) * s, oy: rnd(-1.0, 1.0) * s };
        let b = biome(self.wave);
        let t = self.t;

        clear_background(rgba(0.05, 0.04, 0.08, 1.0));
        draw_sky(&g0, self.lw, t, &b);
        draw_ambient(&g0, self.lw, t, &b);
        draw_ground(&gs, self.lw, t, &b);
        draw_castle(&gs, t, self.hp / self.hp_max());

        for e in self.enemies.iter() {
            draw_enemy(&gs, e, t);
        }

        // enemy shots
        for ep in self.eprojs.iter() {
            let d = ep.v.normalize();
            let tail = ep.p - d * 16.0;
            let c = match ep.kind {
                1 => rgba(1.0, 0.45, 0.1, 1.0),
                2 => rgba(0.6, 0.3, 1.0, 1.0),
                _ => rgba(0.7, 1.0, 0.4, 1.0),
            };
            gs.line(tail.x, tail.y, ep.p.x, ep.p.y, 5.0, with_a(c, 0.4));
            gs.glow(ep.p.x, ep.p.y, 12.0, c);
            gs.circ(ep.p.x, ep.p.y, 4.0, WHITE);
        }

        // hero
        if self.rage > 0.0 {
            let o = hero_origin();
            gs.glow(o.x, o.y + 10.0, 45.0, rgba(1.0, 0.4, 0.15, 0.8));
        }
        draw_hero_at(&gs, self.hero, 80.0, GROUND - CASTLE_H, t);
        let ready = 1.0 - (self.shoot_cd / self.cooldown()).clamp(0.0, 1.0);
        draw_weapon_at(&gs, self.weapon, hero_origin(), self.aim, self.recoil, ready, t);
        if self.muzzle > 0.0 {
            let tip = hero_origin() + self.aim * 40.0;
            gs.glow(tip.x, tip.y, 22.0 * self.muzzle, rgba(1.0, 0.85, 0.4, 0.9));
        }

        // projectiles
        for p in self.projs.iter() {
            let d = p.v.normalize();
            match p.kind {
                0 => {
                    let tail = p.p - d * 28.0;
                    gs.line(tail.x, tail.y, p.p.x, p.p.y, 7.0, rgba(1.0, 0.7, 0.2, 0.22));
                    gs.line(tail.x, tail.y, p.p.x, p.p.y, 2.6, rgba(1.0, 0.97, 0.8, 1.0));
                    gs.circ(p.p.x, p.p.y, 2.6, WHITE);
                    if p.crit {
                        gs.glow(p.p.x, p.p.y, 14.0, rgba(1.0, 0.85, 0.2, 0.9));
                    }
                }
                1 => {
                    let tail = p.p - d * 40.0;
                    gs.line(tail.x, tail.y, p.p.x, p.p.y, 9.0, rgba(0.5, 0.8, 1.0, 0.25));
                    gs.line(tail.x, tail.y, p.p.x, p.p.y, 4.0, rgba(0.85, 0.9, 1.0, 1.0));
                    gs.circ(p.p.x, p.p.y, 3.5, WHITE);
                    if p.crit {
                        gs.glow(p.p.x, p.p.y, 16.0, rgba(1.0, 0.85, 0.2, 0.9));
                    }
                }
                2 => {
                    gs.glow(p.p.x, p.p.y, 20.0, rgba(1.0, 0.5, 0.1, 0.9));
                    gs.circ(p.p.x, p.p.y, 8.0, rgba(1.0, 0.6, 0.15, 1.0));
                    gs.circ(p.p.x, p.p.y, 4.5, rgba(1.0, 0.95, 0.5, 1.0));
                }
                _ => {
                    gs.circ(p.p.x, p.p.y, 9.0, rgba(0.12, 0.12, 0.16, 1.0));
                    gs.circ(p.p.x - 3.0, p.p.y - 3.0, 3.0, rgba(0.5, 0.5, 0.6, 1.0));
                }
            }
        }

        // lightning
        for bt in self.bolts.iter() {
            let a = (bt.life / 0.2).clamp(0.0, 1.0);
            for w in 0..bt.pts.len().saturating_sub(1) {
                let p = bt.pts[w];
                let q = bt.pts[w + 1];
                gs.line(p.x, p.y, q.x, q.y, 9.0, with_a(bt.c, 0.25 * a));
                gs.line(p.x, p.y, q.x, q.y, 4.0, with_a(bt.c, 0.6 * a));
                gs.line(p.x, p.y, q.x, q.y, 1.8, rgba(1.0, 1.0, 1.0, a));
            }
        }

        // meteors
        for m in self.meteors.iter() {
            let d = m.v.normalize();
            let tail = m.p - d * 60.0;
            gs.line(tail.x, tail.y, m.p.x, m.p.y, 12.0, rgba(1.0, 0.4, 0.1, 0.3));
            gs.glow(m.p.x, m.p.y, 22.0, rgba(1.0, 0.5, 0.1, 0.9));
            gs.circ(m.p.x, m.p.y, 8.0, rgba(1.0, 0.5, 0.1, 1.0));
            gs.circ(m.p.x, m.p.y, 4.5, rgba(1.0, 0.95, 0.5, 1.0));
        }

        // rings
        for r in self.rings.iter() {
            let k = 1.0 - r.life / r.max;
            gs.ring(r.p.x, r.p.y, r.r * (0.2 + 0.8 * k), 4.0 * (1.0 - k) + 1.0, with_a(r.c, 1.0 - k));
        }

        // particles
        for p in self.parts.iter() {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let sz = p.size * (0.5 + a * 0.5);
            gs.rect(p.p.x, p.p.y, sz, sz, with_a(p.c, a));
        }

        // damage numbers
        for f in self.ftexts.iter() {
            let a = (f.life / 0.8).min(1.0);
            if f.crit {
                gs.text_center(&format!("{}!", fmt(f.val as f64)), f.p.x, f.p.y, 28.0, rgba(1.0, 0.85, 0.2, a));
            } else {
                gs.text_center(&fmt(f.val as f64), f.p.x, f.p.y, 18.0, rgba(1.0, 1.0, 1.0, a));
            }
        }

        // screen tints
        if self.flash > 0.0 {
            g0.rect(0.0, 0.0, self.lw, BASE_H, rgba(1.0, 0.0, 0.0, self.flash * 0.3));
        }
        if self.cold > 0.0 {
            g0.rect(0.0, 0.0, self.lw, BASE_H, rgba(0.4, 0.75, 1.0, self.cold * 0.25));
        }

        self.draw_hud(&g0, &b);
        self.draw_buttons(&g0);

        if self.shop {
            self.draw_shop(&g0);
        }

        if self.over {
            let a = (self.over_t * 2.0).min(1.0);
            g0.rect(0.0, 0.0, self.lw, BASE_H, rgba(0.0, 0.0, 0.0, 0.72 * a));
            g0.text_center("GAME OVER", self.lw / 2.0, 190.0, 80.0, rgba(1.0, 0.3, 0.3, a));
            g0.text_center(&format!("Wave {}   Kills {}   Best {}", self.wave, self.kills, self.best), self.lw / 2.0, 250.0, 32.0, rgba(1.0, 1.0, 1.0, a));
            if self.over_t > 0.7 {
                let pulse = 0.6 + 0.4 * (self.t * 5.0).sin();
                g0.text_center("TAP TO PLAY AGAIN", self.lw / 2.0, 320.0, 34.0, rgba(1.0, 0.9, 0.5, pulse));
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Castle Defense".to_owned(),
        window_width: 960,
        window_height: 540,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut g = Game::new();
    loop {
        let dt = get_frame_time().min(0.05);
        let u = screen_height() / BASE_H;
        g.lw = screen_width() / u;
        g.update(dt, u);
        g.draw(u);
        next_frame().await;
    }
}
