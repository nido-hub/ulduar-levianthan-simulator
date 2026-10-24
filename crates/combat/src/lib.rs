//! Headless raid prototype. Values are normalized for training, not certified 3.3.5 data.
use leviathan_driving::{Point, Vehicle};
use std::f32::consts::TAU;

pub fn distance(a: Point, b: Point) -> f32 {
    (a.x - b.x).hypot(a.z - b.z)
}
fn bounded(p: Point) -> Point {
    Point {
        x: p.x.clamp(-51.0, 51.0),
        z: p.z.clamp(-51.0, 51.0),
    }
}
fn toward(a: Point, b: Point, step: f32) -> Point {
    let d = distance(a, b);
    if d < 0.001 {
        return a;
    }
    let t = (step / d).min(1.0);
    bounded(Point {
        x: a.x + (b.x - a.x) * t,
        z: a.z + (b.z - a.z) * t,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Demo,
    Siege,
    Moto,
}
impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Demo => "Démolisseur",
            Self::Siege => "Siège",
            Self::Moto => "Moto",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    DemoDriver,
    DemoPassenger,
    SiegeDriver,
    SiegeGunner,
    Moto,
}
impl Role {
    pub const ALL: [Self; 5] = [
        Self::DemoDriver,
        Self::DemoPassenger,
        Self::SiegeDriver,
        Self::SiegeGunner,
        Self::Moto,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::DemoDriver => "Démolisseur · conducteur",
            Self::DemoPassenger => "Démolisseur · passager",
            Self::SiegeDriver => "Siège · conducteur",
            Self::SiegeGunner => "Siège · tourelle",
            Self::Moto => "Moto · conducteur",
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::DemoDriver | Self::DemoPassenger => 0,
            Self::SiegeDriver | Self::SiegeGunner => 2,
            Self::Moto => 4,
        }
    }
    pub fn drives(self) -> bool {
        matches!(self, Self::DemoDriver | Self::SiegeDriver | Self::Moto)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Spell {
    pub name: &'static str,
    pub aimed: bool,
    pub min: f32,
    pub max: f32,
    pub cost: f32,
    pub cooldown: f32,
    pub radius: f32,
}
const fn spell(
    name: &'static str,
    aimed: bool,
    min: f32,
    max: f32,
    cost: f32,
    cooldown: f32,
    radius: f32,
) -> Spell {
    Spell {
        name,
        aimed,
        min,
        max,
        cost,
        cooldown,
        radius,
    }
}
pub fn spells(role: Role) -> Vec<Spell> {
    match role {
        Role::DemoDriver => vec![
            spell("Rocher", true, 8., 70., 0., 1.5, 4.),
            spell("Pyrite", true, 8., 70., 5., 1., 3.),
            spell("Bélier", false, 0., 9., 0., 4., 3.),
            spell("Projeter passager", true, 8., 70., 0., 2., 4.),
        ],
        Role::DemoPassenger => vec![
            spell("Mortier", true, 0., 50., 0., 1.5, 4.),
            spell("Roquette anti-air", true, 0., 60., 0., 1., 3.),
            spell("Saisir une caisse", true, 0., 25., 0., 2., 3.),
            spell("Accélération", false, 0., 0., 25., 20., 0.),
            spell("Charger catapulte", false, 0., 0., 0., 30., 0.),
        ],
        Role::SiegeDriver => vec![
            spell("Bélier", false, 0., 9., 15., 2., 3.),
            spell("Électrochoc", false, 0., 18., 30., 10., 3.),
            spell("Ruée vapeur", false, 0., 0., 20., 15., 0.),
        ],
        Role::SiegeGunner => vec![
            spell("Roquette anti-air", true, 0., 60., 10., 1., 3.),
            spell("Canon", true, 0., 50., 20., 2., 4.),
            spell("Bouclier", false, 0., 0., 0., 30., 0.),
        ],
        Role::Moto => vec![
            spell("Cor sonore", false, 0., 18., 20., 2., 4.),
            spell("Goudron", false, 0., 0., 0., 15., 7.),
            spell("Accélération", false, 0., 0., 50., 15., 0.),
            spell("Secours / déposer", false, 0., 7., 0., 0.5, 0.),
            spell("Premiers soins", false, 0., 0., 0., 15., 0.),
        ],
    }
}
#[derive(Clone, Debug)]
pub struct CrewVehicle {
    pub kind: Kind,
    pub body: Vehicle,
    pub hp: f32,
    pub energy: f32,
    pub gun_energy: f32,
    pub pyrite: f32,
    pub cooldowns: [f32; 5],
    pub ai_cd: [f32; 5],
    pub boost: f32,
    pub shield: f32,
    pub frozen: f32,
    pub stacks: u8,
    pub stack_time: f32,
    pub passenger: bool,
    pub loaded: bool,
    pub cargo: Option<u64>,
    pub ai_status: &'static str,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotKind {
    Boulder,
    Pyrite,
    Mortar,
    Cannon,
    Rocket,
    Passenger,
}
#[derive(Clone, Debug)]
pub struct Shot {
    pub id: u64,
    pub owner: usize,
    pub start: Point,
    pub end: Point,
    pub age: f32,
    pub flight: f32,
    pub kind: ShotKind,
    pub personal: bool,
}
#[derive(Clone, Debug)]
pub struct Impact {
    pub id: u64,
    pub position: Point,
    pub age: f32,
    pub hit: bool,
    pub height: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HazardKind {
    Storm,
    Fire,
    Frost,
    Tar,
}
#[derive(Clone, Debug)]
pub struct Hazard {
    pub id: u64,
    pub kind: HazardKind,
    pub position: Point,
    pub radius: f32,
    pub age: f32,
    pub duration: f32,
    pub burning: bool,
    pub target: usize,
}
#[derive(Clone, Debug)]
pub struct Crate {
    pub id: u64,
    pub position: Point,
    pub air: f32,
    pub life: f32,
}
#[derive(Clone, Debug)]
pub struct Add {
    pub id: u64,
    pub position: Point,
    pub hp: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiderState {
    Turret,
    Ground,
    Carried,
    Lost,
}
#[derive(Clone, Debug)]
pub struct Rider {
    pub id: u64,
    pub home: usize,
    pub position: Point,
    pub state: RiderState,
    pub hp: f32,
}
#[derive(Clone, Debug)]
pub struct Boss {
    pub position: Point,
    pub hp: f32,
    pub target: usize,
    pub pursuit: f32,
    pub cast: f32,
    pub vents: f32,
    pub shutdown: f32,
    pub turret_timer: f32,
    pub missile: f32,
    pub travelled: f32,
}
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub shots: u32,
    pub hits: u32,
    pub misses: u32,
    pub interrupts: u32,
    pub vents_missed: u32,
    pub stack_drops: u32,
    pub rescued: u32,
    pub avoidable: f32,
    pub shutdowns: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ready,
    Fighting,
    Victory,
    Defeat,
}
pub struct Fight {
    pub role: Role,
    pub vehicles: Vec<CrewVehicle>,
    pub boss: Boss,
    pub shots: Vec<Shot>,
    pub impacts: Vec<Impact>,
    pub hazards: Vec<Hazard>,
    pub crates: Vec<Crate>,
    pub adds: Vec<Add>,
    pub riders: Vec<Rider>,
    pub time: f32,
    pub outcome: Outcome,
    pub towers: bool,
    pub log: Vec<String>,
    pub stats: Stats,
    next_id: u64,
    crate_timer: f32,
    storm_timer: f32,
    fire_timer: f32,
    fire_phase: u32,
    frost_timer: f32,
    life_timer: f32,
    launch_timer: f32,
}
impl Fight {
    pub fn new(role: Role, towers: bool) -> Self {
        let vehicles = [
            (Kind::Demo, -18., 30.),
            (Kind::Demo, 18., 30.),
            (Kind::Siege, -12., 4.),
            (Kind::Siege, 12., 4.),
            (Kind::Moto, -30., 22.),
            (Kind::Moto, 30., 22.),
        ]
        .into_iter()
        .map(|(kind, x, z)| CrewVehicle {
            kind,
            body: Vehicle {
                position: Point { x, z },
                ..Default::default()
            },
            hp: 100.,
            energy: 100.,
            gun_energy: 100.,
            pyrite: 100.,
            cooldowns: [0.; 5],
            ai_cd: [0.; 5],
            boost: 0.,
            shield: 0.,
            frozen: 0.,
            stacks: 0,
            stack_time: 0.,
            passenger: kind != Kind::Moto,
            loaded: false,
            cargo: None,
            ai_status: "Prêt",
        })
        .collect();
        Self {
            role,
            vehicles,
            boss: Boss {
                position: Point { x: 0., z: -17. },
                hp: 1000.,
                target: 2,
                pursuit: 0.,
                cast: 0.,
                vents: 12.,
                shutdown: 0.,
                turret_timer: 0.,
                missile: 5.,
                travelled: 0.,
            },
            shots: vec![],
            impacts: vec![],
            hazards: vec![],
            crates: vec![],
            adds: vec![],
            riders: vec![],
            time: 0.,
            outcome: Outcome::Ready,
            towers,
            log: vec!["Entrée : lancer le combat. F2 : changer de rôle avant le pull.".into()],
            stats: Stats::default(),
            next_id: 1,
            crate_timer: 1.,
            storm_timer: 8.,
            fire_timer: 15.,
            fire_phase: 0,
            frost_timer: 24.,
            life_timer: 30.,
            launch_timer: 48.,
        }
    }
    fn id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub fn player(&self) -> &CrewVehicle {
        &self.vehicles[self.role.index()]
    }
    pub fn player_mut(&mut self) -> &mut CrewVehicle {
        &mut self.vehicles[self.role.index()]
    }
    pub fn say(&mut self, text: impl Into<String>) {
        self.log.insert(
            0,
            format!(
                "{:02}:{:02} {}",
                self.time as u32 / 60,
                self.time as u32 % 60,
                text.into()
            ),
        );
        self.log.truncate(8);
    }
    pub fn start(&mut self) {
        if self.outcome == Outcome::Ready {
            self.outcome = Outcome::Fighting;
            self.say("Pull : le Léviathan s'active.");
        }
    }
    pub fn resource(&self) -> f32 {
        let v = self.player();
        match self.role {
            Role::DemoDriver | Role::DemoPassenger => v.pyrite,
            Role::SiegeGunner => v.gun_energy,
            _ => v.energy,
        }
    }
    pub fn onboard(&self) -> bool {
        self.role != Role::DemoPassenger || self.player().passenger
    }
    pub fn aim_valid(&self, index: usize, point: Point) -> bool {
        let Some(spell) = spells(self.role).get(index).copied() else {
            return false;
        };
        let d = distance(self.player().body.position, point);
        point.x.is_finite()
            && point.z.is_finite()
            && point.x.abs() <= 55.
            && point.z.abs() <= 55.
            && d >= spell.min
            && d <= spell.max
    }
    pub fn cast(&mut self, index: usize, aim: Option<Point>) -> Result<(), &'static str> {
        if self.outcome != Outcome::Fighting {
            return Err("Lance le combat avec Entrée.");
        }
        let Some(spell) = spells(self.role).get(index).copied() else {
            return Err("Pas de sort sur cette touche.");
        };
        let owner = self.role.index();
        let v = &self.vehicles[owner];
        if !self.onboard() {
            return Err("Passager hors véhicule : retour par une moto en cours.");
        }
        if v.hp <= 0. || v.frozen > 0. {
            return Err("Véhicule détruit ou gelé.");
        }
        if v.cooldowns[index] > 0. {
            return Err("Sort en recharge.");
        }
        if self.resource() < spell.cost {
            return Err("Ressource insuffisante.");
        }
        if spell.aimed && !aim.is_some_and(|p| self.aim_valid(index, p)) {
            return Err("Visée hors portée ou hors de la cour.");
        }
        // Preconditions must be checked before consuming any resource or cooldown.
        match (self.role, index) {
            (Role::DemoDriver, 3) if !v.passenger || !v.loaded => {
                return Err("Le passager doit être chargé dans la catapulte.");
            }
            (Role::DemoPassenger, 2)
                if !self
                    .crates
                    .iter()
                    .any(|c| c.air <= 0. && distance(c.position, aim.unwrap()) < 4.) =>
            {
                return Err("Vise une caisse au sol.");
            }
            (Role::DemoPassenger, 2) if v.pyrite >= 100. => {
                return Err("Réserve de pyrite pleine.");
            }
            (Role::Moto, 3) => {
                if let Some(id) = v.cargo {
                    let r = self.riders.iter().find(|r| r.id == id).unwrap();
                    if self.vehicles[r.home].hp <= 0.
                        || distance(v.body.position, self.vehicles[r.home].body.position) > 8.
                    {
                        return Err("Rejoins le démolisseur d'origine avant de déposer.");
                    }
                } else if !self.riders.iter().any(|r| {
                    r.state == RiderState::Ground && distance(r.position, v.body.position) < 7.
                }) {
                    return Err("Aucun joueur au sol à proximité.");
                }
            }
            (Role::Moto, 4) if v.cargo.is_none() => return Err("Aucun passager à soigner."),
            _ => {}
        }
        let v = &mut self.vehicles[owner];
        v.cooldowns[index] = spell.cooldown;
        match self.role {
            Role::DemoDriver | Role::DemoPassenger => v.pyrite -= spell.cost,
            Role::SiegeGunner => v.gun_energy -= spell.cost,
            _ => v.energy -= spell.cost,
        }
        match (self.role, index) {
            (Role::DemoDriver, 0) => self.fire(owner, aim.unwrap(), ShotKind::Boulder, true),
            (Role::DemoDriver, 1) => self.fire(owner, aim.unwrap(), ShotKind::Pyrite, true),
            (Role::DemoDriver, 2) | (Role::SiegeDriver, 0) => self.frontal(owner, 9., 12.),
            (Role::DemoDriver, 3) => {
                self.vehicles[owner].passenger = false;
                self.vehicles[owner].loaded = false;
                self.fire(owner, aim.unwrap(), ShotKind::Passenger, true);
            }
            (Role::DemoPassenger, 0) => self.fire(owner, aim.unwrap(), ShotKind::Mortar, true),
            (Role::DemoPassenger, 1) | (Role::SiegeGunner, 0) => {
                self.fire(owner, aim.unwrap(), ShotKind::Rocket, true)
            }
            (Role::DemoPassenger, 2) => {
                let point = aim.unwrap();
                if let Some(id) = self
                    .crates
                    .iter()
                    .find(|c| c.air <= 0. && distance(c.position, point) < 4.)
                    .map(|c| c.id)
                {
                    self.crates.retain(|c| c.id != id);
                }
                self.vehicles[owner].pyrite = (self.vehicles[owner].pyrite + 25.).min(100.);
            }
            (Role::DemoPassenger, 3) | (Role::SiegeDriver, 2) | (Role::Moto, 2) => {
                self.vehicles[owner].boost = 5.
            }
            (Role::DemoPassenger, 4) => {
                self.vehicles[owner].loaded = true;
                self.say("Chargé : le conducteur IA prépare ta projection.");
            }
            (Role::SiegeDriver, 1) => {
                if self.in_front(owner, self.boss.position, 18.) && self.boss.cast > 0. {
                    self.boss.cast = 0.;
                    self.stats.interrupts += 1;
                    self.say("Électrochoc : interruption réussie.");
                } else {
                    self.say("Électrochoc : pas d'incantation à portée dans le cône.");
                }
            }
            (Role::SiegeGunner, 1) => self.fire(owner, aim.unwrap(), ShotKind::Cannon, true),
            (Role::SiegeGunner, 2) => self.vehicles[owner].shield = 5.,
            (Role::Moto, 0) => self.frontal(owner, 18., 9.),
            (Role::Moto, 1) => {
                let position = self.vehicles[owner].body.position;
                let id = self.id();
                self.hazards.push(Hazard {
                    id,
                    kind: HazardKind::Tar,
                    position,
                    radius: 7.,
                    age: 0.,
                    duration: 35.,
                    burning: false,
                    target: owner,
                });
            }
            (Role::Moto, 3) => {
                self.rescue(owner);
            }
            (Role::Moto, 4) => {
                let id = self.vehicles[owner].cargo.unwrap();
                if let Some(r) = self.riders.iter_mut().find(|r| r.id == id) {
                    r.hp = 100.;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn fire(&mut self, owner: usize, end: Point, kind: ShotKind, personal: bool) {
        let start = self.vehicles[owner].body.position;
        let id = self.id();
        self.shots.push(Shot {
            id,
            owner,
            start,
            end,
            age: 0.,
            flight: (distance(start, end) / 38.).max(0.35),
            kind,
            personal,
        });
        if personal && kind != ShotKind::Passenger {
            self.stats.shots += 1;
        }
    }
    fn in_front(&self, owner: usize, point: Point, range: f32) -> bool {
        let v = &self.vehicles[owner];
        let dx = point.x - v.body.position.x;
        let dz = point.z - v.body.position.z;
        let d = dx.hypot(dz);
        d <= range
            && (d < 0.1 || (dx * v.body.heading.sin() - dz * v.body.heading.cos()) / d > 0.55)
    }
    fn frontal(&mut self, owner: usize, range: f32, damage: f32) {
        if self.in_front(owner, self.boss.position, range) {
            self.damage_boss(damage);
        }
        let hits: Vec<_> = self
            .adds
            .iter()
            .filter(|a| self.in_front(owner, a.position, range))
            .map(|a| a.id)
            .collect();
        for a in &mut self.adds {
            if hits.contains(&a.id) {
                a.hp -= damage * 2.;
            }
        }
    }
    fn damage_boss(&mut self, damage: f32) {
        self.boss.hp =
            (self.boss.hp - damage * if self.boss.shutdown > 0. { 1.5 } else { 1. }).max(0.);
    }
    fn harm(&mut self, index: usize, damage: f32, avoidable: bool) {
        let v = &mut self.vehicles[index];
        let damage = damage * if v.shield > 0. { 0.2 } else { 1. };
        let actual = damage.min(v.hp);
        v.hp = (v.hp - damage).max(0.);
        if index == self.role.index() && avoidable {
            self.stats.avoidable += actual;
        }
    }
    fn impact(&mut self, shot: Shot) {
        let radius = match shot.kind {
            ShotKind::Pyrite => 3.,
            ShotKind::Rocket => 3.,
            _ => 4.,
        };
        let boss_hit = distance(shot.end, self.boss.position) < radius + 4.5;
        if shot.kind == ShotKind::Passenger {
            let id = self.id();
            self.riders.push(Rider {
                id,
                home: shot.owner,
                position: shot.end,
                state: if boss_hit {
                    RiderState::Turret
                } else {
                    RiderState::Ground
                },
                hp: 70.,
            });
            self.say(if boss_hit {
                "Passager sur les tourelles."
            } else {
                "Projection manquée : passager au sol."
            });
            return;
        }
        let mut hit = false;
        if shot.kind == ShotKind::Rocket {
            for c in &mut self.crates {
                if c.air > 0. && distance(c.position, shot.end) < radius {
                    c.air = 0.;
                    hit = true;
                }
            }
        } else {
            if boss_hit {
                hit = true;
                self.damage_boss(match shot.kind {
                    ShotKind::Pyrite => 3.,
                    ShotKind::Boulder => 5.,
                    _ => 6.,
                });
                if shot.kind == ShotKind::Pyrite {
                    let v = &mut self.vehicles[shot.owner];
                    v.stacks = (v.stacks + 1).min(10);
                    v.stack_time = 10.;
                }
            }
            for a in &mut self.adds {
                if distance(a.position, shot.end) < radius + 1. {
                    a.hp -= 20.;
                    hit = true;
                }
            }
            for h in &mut self.hazards {
                if h.kind == HazardKind::Tar && distance(h.position, shot.end) < h.radius + radius {
                    h.burning = true;
                }
            }
            for v in &mut self.vehicles {
                if v.frozen > 0. && distance(v.body.position, shot.end) < radius + 2. {
                    v.frozen = 0.;
                    hit = true;
                }
            }
        }
        if shot.personal {
            if hit {
                self.stats.hits += 1;
            } else {
                self.stats.misses += 1;
                self.say("Tir manqué : le projectile touche le sol.");
            }
        }
        let id = self.id();
        self.impacts.push(Impact {
            id,
            position: shot.end,
            age: 0.,
            hit,
            height: if shot.kind == ShotKind::Rocket {
                8.
            } else {
                0.15
            },
        });
    }
    fn rescue(&mut self, moto: usize) -> bool {
        let position = self.vehicles[moto].body.position;
        if let Some(id) = self.vehicles[moto].cargo {
            let Some(r) = self.riders.iter().find(|r| r.id == id) else {
                return false;
            };
            let home = r.home;
            if self.vehicles[home].hp <= 0. {
                self.vehicles[moto].cargo = None;
                if let Some(r) = self.riders.iter_mut().find(|r| r.id == id) {
                    r.state = RiderState::Lost;
                }
                return false;
            }
            if distance(position, self.vehicles[home].body.position) < 8. {
                self.vehicles[home].passenger = true;
                self.vehicles[moto].cargo = None;
                self.riders.retain(|r| r.id != id);
                if moto == self.role.index() {
                    self.stats.rescued += 1;
                }
                self.say("Passager ramené à son démolisseur.");
                return true;
            }
        } else if let Some(r) = self
            .riders
            .iter_mut()
            .find(|r| r.state == RiderState::Ground && distance(r.position, position) < 7.)
        {
            r.state = RiderState::Carried;
            self.vehicles[moto].cargo = Some(r.id);
            return true;
        }
        false
    }
    fn ai(&mut self, index: usize, dt: f32) {
        if self.vehicles[index].hp <= 0. || self.vehicles[index].frozen > 0. {
            return;
        }
        let v = self.vehicles[index].clone();
        let is_player = index == self.role.index();
        let driver = !is_player || !self.role.drives();
        if driver {
            self.vehicles[index].body.velocity = Point::default();
            let mut goal;
            if v.kind == Kind::Moto {
                if let Some(id) = v.cargo {
                    goal = self
                        .riders
                        .iter()
                        .find(|r| r.id == id)
                        .map(|r| self.vehicles[r.home].body.position)
                        .unwrap_or(Point { x: 0., z: 35. });
                } else {
                    let reserved: Vec<_> = self
                        .vehicles
                        .iter()
                        .filter(|v| v.cargo.is_some())
                        .filter_map(|v| v.cargo)
                        .collect();
                    goal = self
                        .riders
                        .iter()
                        .filter(|r| r.state == RiderState::Ground && !reserved.contains(&r.id))
                        .min_by(|a, b| {
                            distance(a.position, v.body.position)
                                .total_cmp(&distance(b.position, v.body.position))
                        })
                        .map(|r| r.position)
                        .unwrap_or(Point {
                            x: 42. * (self.time * 0.12 + index as f32).sin(),
                            z: 42. * (self.time * 0.12 + index as f32).cos(),
                        });
                }
            } else if self.boss.target == index && self.boss.shutdown <= 0. {
                let angle = v.body.position.z.atan2(v.body.position.x) + 0.4;
                goal = Point {
                    x: 46. * angle.cos(),
                    z: 46. * angle.sin(),
                };
                if distance(v.body.position, self.boss.position) < 22.
                    && v.ai_cd[4] <= 0.
                    && v.energy >= 20.
                {
                    self.vehicles[index].boost = 5.;
                    self.vehicles[index].ai_cd[4] = 15.;
                    self.vehicles[index].energy -= 20.;
                }
            } else {
                let radius = if v.kind == Kind::Demo { 32. } else { 14. };
                let base = (v.body.position.z - self.boss.position.z)
                    .atan2(v.body.position.x - self.boss.position.x);
                goal = Point { x: 0., z: 0. };
                let mut best = f32::INFINITY;
                for step in 0..24 {
                    let a = base + step as f32 * TAU / 24.;
                    let p = Point {
                        x: self.boss.position.x + radius * a.cos(),
                        z: self.boss.position.z + radius * a.sin(),
                    };
                    if p.x.abs() > 50. || p.z.abs() > 50. {
                        continue;
                    }
                    let d = distance(p, v.body.position);
                    if d < best {
                        best = d;
                        goal = p;
                    }
                }
            }
            // Choose an inset alternative when a hazard would trap a vehicle against a wall.
            if self.hazards.iter().any(|h| {
                h.kind != HazardKind::Tar
                    && h.age >= 0.
                    && distance(v.body.position, h.position) < h.radius + 3.
            }) {
                let mut best = -f32::INFINITY;
                for step in 0..16 {
                    let a = step as f32 * TAU / 16.;
                    let p = bounded(Point {
                        x: v.body.position.x + 12. * a.cos(),
                        z: v.body.position.z + 12. * a.sin(),
                    });
                    let clearance = self
                        .hazards
                        .iter()
                        .filter(|h| h.kind != HazardKind::Tar)
                        .map(|h| distance(p, h.position) - h.radius)
                        .fold(100., f32::min)
                        - distance(p, goal) * 0.03;
                    if clearance > best {
                        best = clearance;
                        goal = p;
                    }
                }
            }
            let speed = match v.kind {
                Kind::Demo => 9.,
                Kind::Siege => 10.,
                Kind::Moto => 14.,
            } * if v.boost > 0. { 1.7 } else { 1. };
            let stop = if v.kind == Kind::Moto { 3. } else { 1.5 };
            let old = v.body.position;
            if distance(old, goal) > stop {
                let next = toward(old, goal, speed * dt);
                self.vehicles[index].body.position = next;
                self.vehicles[index].body.velocity = Point {
                    x: (next.x - old.x) / dt,
                    z: (next.z - old.z) / dt,
                };
                self.vehicles[index].body.distance += distance(old, next);
                self.vehicles[index].body.heading = (next.x - old.x).atan2(-(next.z - old.z));
            } else if v.kind != Kind::Moto && self.boss.target != index {
                self.vehicles[index].body.heading =
                    (self.boss.position.x - old.x).atan2(-(self.boss.position.z - old.z));
            }
            self.vehicles[index].ai_status = if self.boss.target == index {
                "Fuite"
            } else if v.kind == Kind::Moto {
                "Secours / patrouille"
            } else {
                "Placement"
            };
            if v.kind == Kind::Moto {
                self.rescue(index);
            }
        }
        let pos = self.vehicles[index].body.position;
        let d = distance(pos, self.boss.position);
        // Driver and passenger automation are independent; never fire the player's spells.
        if v.kind == Kind::Demo {
            let ai_passenger = !is_player || self.role != Role::DemoPassenger;
            if ai_passenger && v.passenger && v.pyrite <= 75. && v.ai_cd[2] <= 0. {
                if let Some(c) = self
                    .crates
                    .iter()
                    .find(|c| c.air <= 0. && distance(c.position, pos) < 25.)
                    .map(|c| c.id)
                {
                    self.crates.retain(|a| a.id != c);
                    self.vehicles[index].pyrite = (v.pyrite + 25.).min(100.);
                    self.vehicles[index].ai_cd[2] = 3.;
                }
            }
            if ai_passenger && v.passenger && self.time > 35. {
                self.vehicles[index].loaded = true;
            }
            if driver && v.ai_cd[0] <= 0. && d < 68. && d > 8. {
                let kind = if v.pyrite >= 5. && (v.stacks < 10 || v.stack_time < 3.) {
                    self.vehicles[index].pyrite -= 5.;
                    ShotKind::Pyrite
                } else {
                    ShotKind::Boulder
                };
                self.fire(index, self.boss.position, kind, false);
                self.vehicles[index].ai_cd[0] = 1.6;
            }
            if is_player
                && self.role == Role::DemoPassenger
                && v.loaded
                && v.passenger
                && v.ai_cd[3] <= 0.
                && d < 65.
            {
                self.vehicles[index].passenger = false;
                self.vehicles[index].loaded = false;
                self.vehicles[index].ai_cd[3] = 35.;
                self.fire(index, self.boss.position, ShotKind::Passenger, false);
            }
        } else if v.kind == Kind::Siege {
            if driver
                && self.boss.cast > 0.
                && self.boss.cast < 1.6
                && d < 18.
                && v.ai_cd[1] <= 0.
                && v.energy >= 30.
            {
                self.boss.cast = 0.;
                self.vehicles[index].energy -= 30.;
                self.vehicles[index].ai_cd[1] = 10.;
                self.say("Électrochoc allié : interruption.");
            }
            if (!is_player || self.role != Role::SiegeGunner) && v.ai_cd[0] <= 0. && d < 48. {
                self.fire(index, self.boss.position, ShotKind::Cannon, false);
                self.vehicles[index].ai_cd[0] = 3.;
            }
        } else if v.ai_cd[0] <= 0. {
            if let Some(add) = self
                .adds
                .iter_mut()
                .find(|a| distance(a.position, pos) < 18.)
            {
                add.hp -= 10.;
            }
            if let Some(id) = v.cargo
                && let Some(r) = self.riders.iter_mut().find(|r| r.id == id)
            {
                r.hp = (r.hp + 25.).min(100.);
            }
            self.vehicles[index].ai_cd[0] = 3.;
        }
    }
    pub fn tick(&mut self, dt: f32) {
        if self.outcome != Outcome::Fighting || !dt.is_finite() || dt <= 0. || dt > 0.1 {
            return;
        }
        self.time += dt;
        for v in &mut self.vehicles {
            for cd in v.cooldowns.iter_mut().chain(v.ai_cd.iter_mut()) {
                *cd = (*cd - dt).max(0.);
            }
            v.energy = (v.energy + 10. * dt).min(100.);
            v.gun_energy = (v.gun_energy + 10. * dt).min(100.);
            for timer in [&mut v.boost, &mut v.shield, &mut v.frozen] {
                *timer = (*timer - dt).max(0.);
            }
        }
        for i in 0..6 {
            self.ai(i, dt);
        }
        for i in 0..6 {
            let stacks = self.vehicles[i].stacks;
            if stacks > 0 {
                self.damage_boss(stacks as f32 * 0.55 * dt);
                self.vehicles[i].stack_time -= dt;
                if self.vehicles[i].stack_time <= 0. {
                    self.vehicles[i].stacks = 0;
                    if i == self.role.index() {
                        self.stats.stack_drops += 1;
                        self.say("Charges de pyrite perdues.");
                    }
                }
            }
        }
        if self.boss.shutdown > 0. {
            self.boss.shutdown = (self.boss.shutdown - dt).max(0.);
            if self.boss.shutdown == 0. {
                self.boss.pursuit = 0.;
            }
        } else {
            self.boss.pursuit -= dt;
            if self.boss.pursuit <= 0. || self.vehicles[self.boss.target].hp <= 0. {
                let candidates: Vec<_> = (0..4)
                    .filter(|i| self.vehicles[*i].hp > 0. && *i != self.boss.target)
                    .collect();
                if !candidates.is_empty() {
                    self.boss.target = candidates[(self.time as usize / 30) % candidates.len()];
                }
                self.boss.pursuit = 30.;
                self.say(format!(
                    "Poursuite : {} {}",
                    self.vehicles[self.boss.target].kind.name(),
                    self.boss.target % 2 + 1
                ));
            }
            let slow = self.hazards.iter().any(|h| {
                h.kind == HazardKind::Tar
                    && distance(h.position, self.boss.position) < h.radius + 4.
            });
            let target = self.vehicles[self.boss.target].body.position;
            let old_boss = self.boss.position;
            self.boss.position = toward(
                self.boss.position,
                target,
                (5.5 + (30. - self.boss.pursuit) * 0.06) * dt * if slow { 0.5 } else { 1. },
            );
            self.boss.travelled += distance(old_boss, self.boss.position);
            for i in 0..6 {
                if distance(self.vehicles[i].body.position, self.boss.position) < 7.5 {
                    self.harm(i, 14. * dt, true);
                }
            }
            self.boss.vents -= dt;
            if self.boss.vents <= 0. {
                self.boss.vents = 17.;
                self.boss.cast = 3.;
                self.say("Décharges de flammes : interrompre !");
            }
            if self.boss.cast > 0. {
                self.boss.cast -= dt;
                if self.boss.cast <= 0. {
                    self.stats.vents_missed += 1;
                    for i in 0..6 {
                        if distance(self.vehicles[i].body.position, self.boss.position) < 28. {
                            self.harm(i, 12., true);
                        }
                    }
                    self.say("Décharges non interrompues.");
                }
            }
            self.boss.missile -= dt;
            if self.boss.missile <= 0. {
                self.boss.missile = 5.;
                for i in 0..6 {
                    self.harm(i, 0.7, false);
                }
            }
        }
        self.crate_timer -= dt;
        if self.crate_timer <= 0. {
            self.crate_timer = 4.;
            if self.crates.len() < 20 {
                let home = (self.time as usize / 4) % 2;
                let p = self.vehicles[home].body.position;
                let a = self.time * 1.7;
                let position = bounded(Point {
                    x: p.x + 14. * a.cos(),
                    z: p.z + 14. * a.sin(),
                });
                let id = self.id();
                self.crates.push(Crate {
                    id,
                    position,
                    air: 10.,
                    life: 60.,
                });
            }
        }
        for c in &mut self.crates {
            c.air = (c.air - dt).max(0.);
            c.life -= dt;
        }
        self.crates.retain(|c| c.life > 0.);
        // Non-player gunners shoot down supplies after a short delay, not instant refills.
        if self.time as i32 % 5 == 0 {
            for c in &mut self.crates {
                if c.air > 0. && c.air < 7. {
                    c.air = 0.;
                }
            }
        }
        self.launch_timer -= dt;
        if self.launch_timer <= 0. {
            self.launch_timer = 60.;
            for i in 0..2 {
                if i != self.role.index()
                    && self.vehicles[i].passenger
                    && distance(self.vehicles[i].body.position, self.boss.position) < 65.
                {
                    self.vehicles[i].passenger = false;
                    self.fire(i, self.boss.position, ShotKind::Passenger, false);
                }
            }
        }
        let turrets = self
            .riders
            .iter()
            .filter(|r| r.state == RiderState::Turret)
            .count();
        if turrets >= 2 && self.boss.turret_timer == 0. {
            self.boss.turret_timer = 8.;
            self.say("Deux passagers sur le boss : neutralisation des tourelles (automatisée).");
        }
        if self.boss.turret_timer > 0. {
            self.boss.turret_timer -= dt;
            if self.boss.turret_timer <= 0. {
                self.boss.turret_timer = 0.;
                self.boss.shutdown = 20.;
                self.boss.cast = 0.;
                self.stats.shutdowns += 1;
                for (i, r) in self.riders.iter_mut().enumerate() {
                    if r.state == RiderState::Turret {
                        r.state = RiderState::Ground;
                        r.position = bounded(Point {
                            x: self.boss.position.x + if i % 2 == 0 { -10. } else { 10. },
                            z: self.boss.position.z + 10.,
                        });
                    }
                }
                self.say("Surcharge : 20 secondes, récupérer les passagers !");
            }
        }
        for r in &mut self.riders {
            if r.state == RiderState::Ground {
                r.hp -= dt * 0.6;
                if r.hp <= 0. {
                    r.state = RiderState::Lost;
                }
            } else if r.state == RiderState::Turret {
                r.position = self.boss.position;
            }
        }
        for v in &self.vehicles {
            if let Some(id) = v.cargo
                && let Some(r) = self.riders.iter_mut().find(|r| r.id == id)
            {
                r.position = v.body.position;
            }
        }
        if self.towers {
            self.tower_tick(dt);
        }
        for h in &mut self.hazards {
            h.age += dt;
            if h.kind == HazardKind::Frost && h.age < 4.5 {
                h.position = toward(h.position, self.vehicles[h.target].body.position, 6. * dt);
            }
        }
        for i in 0..6 {
            let p = self.vehicles[i].body.position;
            let fire = self.hazards.iter().any(|h| {
                h.kind == HazardKind::Fire && h.age > 0.8 && distance(h.position, p) < h.radius
            });
            if fire {
                self.harm(i, 5. * dt, true);
                self.vehicles[i].frozen = 0.;
            }
            let hits: Vec<_> = self
                .hazards
                .iter()
                .filter(|h| {
                    distance(h.position, p) < h.radius + 1.
                        && ((h.kind == HazardKind::Storm && h.age >= 2.8 && h.age - dt < 2.8)
                            || (h.kind == HazardKind::Frost && h.age >= 6. && h.age - dt < 6.))
                })
                .map(|h| h.kind)
                .collect();
            for kind in hits {
                self.harm(i, 12., true);
                if kind == HazardKind::Frost {
                    self.vehicles[i].frozen = 6.;
                }
            }
        }
        for h in &self.hazards {
            if h.kind == HazardKind::Tar && h.burning {
                for a in &mut self.adds {
                    if distance(a.position, h.position) < h.radius {
                        a.hp -= 12. * dt;
                    }
                }
            }
        }
        self.hazards.retain(|h| h.age < h.duration);
        let old = std::mem::take(&mut self.shots);
        for mut shot in old {
            shot.age += dt;
            if shot.age >= shot.flight {
                self.impact(shot);
            } else {
                self.shots.push(shot);
            }
        }
        for impact in &mut self.impacts {
            impact.age += dt;
        }
        self.impacts.retain(|i| i.age < 0.7);
        let mut damage = [0.; 6];
        for a in &mut self.adds {
            if let Some((i, v)) = self
                .vehicles
                .iter()
                .enumerate()
                .filter(|(_, v)| v.hp > 0.)
                .min_by(|(_, a1), (_, a2)| {
                    distance(a1.body.position, a.position)
                        .total_cmp(&distance(a2.body.position, a.position))
                })
            {
                let slow = self.hazards.iter().any(|h| {
                    h.kind == HazardKind::Tar && distance(h.position, a.position) < h.radius
                });
                a.position = toward(
                    a.position,
                    v.body.position,
                    dt * if slow { 1.5 } else { 4. },
                );
                if distance(a.position, v.body.position) < 3. {
                    damage[i] += dt * 2.;
                }
            }
        }
        for (i, d) in damage.into_iter().enumerate() {
            self.harm(i, d, true);
        }
        self.adds.retain(|a| a.hp > 0.);
        if self.boss.hp <= 0. {
            self.outcome = Outcome::Victory;
            self.say("Victoire ! F5 pour recommencer.");
        } else if self.player().hp <= 0. {
            self.outcome = Outcome::Defeat;
            self.say("Véhicule détruit. F5 pour recommencer.");
        }
    }
    fn tower_tick(&mut self, dt: f32) {
        self.storm_timer -= dt;
        self.fire_timer -= dt;
        self.frost_timer -= dt;
        self.life_timer -= dt;
        if self.storm_timer <= 0. {
            self.storm_timer = 10.;
            for i in [self.role.index(), (self.time as usize) % 6] {
                let id = self.id();
                self.hazards.push(Hazard {
                    id,
                    kind: HazardKind::Storm,
                    position: self.vehicles[i].body.position,
                    radius: 5.,
                    age: 0.,
                    duration: 3.8,
                    burning: false,
                    target: i,
                });
            }
        }
        if self.fire_timer <= 0. {
            self.fire_timer += 0.65;
            let a = self.fire_phase as f32 * TAU / 48.;
            self.fire_phase = (self.fire_phase + 1) % 48;
            let id = self.id();
            self.hazards.push(Hazard {
                id,
                kind: HazardKind::Fire,
                position: Point {
                    x: 29. * a.sin().signum() * a.sin().abs().powf(1.55),
                    z: -29. * a.cos().signum() * a.cos().abs().powf(1.55),
                },
                radius: 2.5,
                age: 0.,
                duration: 34.,
                burning: false,
                target: 0,
            });
        }
        if self.frost_timer <= 0. {
            self.frost_timer = 25.;
            let id = self.id();
            let target = self.role.index();
            self.hazards.push(Hazard {
                id,
                kind: HazardKind::Frost,
                position: bounded(Point {
                    x: self.vehicles[target].body.position.x + (self.time * 0.37).cos() * 16.,
                    z: self.vehicles[target].body.position.z + (self.time * 0.37).sin() * 16.,
                }),
                radius: 4.5,
                age: 0.,
                duration: 7.,
                burning: false,
                target,
            });
        }
        if self.life_timer <= 0. {
            self.life_timer = 30.;
            let sign = if self.time as i32 % 2 == 0 { 1. } else { -1. };
            for i in 0..4 {
                let id = self.id();
                self.adds.push(Add {
                    id,
                    position: Point {
                        x: sign * 46. + i as f32,
                        z: -46.,
                    },
                    hp: 35.,
                });
            }
            self.say("Freya : renforts dans un angle.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frost_ray_can_reach_a_stationary_target() {
        let mut f = playing(Role::DemoDriver);
        f.vehicles[0].hp = 1000.;
        f.towers = true;
        f.frost_timer = 0.;
        f.storm_timer = 100.;
        f.fire_timer = 100.;
        f.life_timer = 100.;
        advance(&mut f, 7.);
        assert!(f.player().frozen > 0.);
    }
    #[test]
    fn death_and_victory_end_the_attempt() {
        let mut f = playing(Role::DemoDriver);
        f.boss.hp = 0.;
        f.tick(0.02);
        assert_eq!(f.outcome, Outcome::Victory);
        let t = f.time;
        f.tick(0.02);
        assert_eq!(f.time, t);
        let mut f = playing(Role::DemoDriver);
        f.vehicles[0].hp = 0.;
        f.tick(0.02);
        assert_eq!(f.outcome, Outcome::Defeat);
    }
    #[test]
    fn overlapping_fire_patches_do_not_multiply_burn_damage() {
        let mut f = playing(Role::DemoDriver);
        let p = f.player().body.position;
        for id in [900, 901] {
            f.hazards.push(Hazard {
                id,
                kind: HazardKind::Fire,
                position: p,
                radius: 3.,
                age: 2.,
                duration: 20.,
                burning: false,
                target: 0,
            });
        }
        f.tick(0.05);
        assert!((f.player().hp - 99.75).abs() < 0.001);
    }
    #[test]
    fn every_role_spell_can_be_cast_with_its_preconditions() {
        for role in Role::ALL {
            for (slot, spec) in spells(role).iter().enumerate() {
                let mut f = playing(role);
                let owner = role.index();
                f.vehicles[owner].body.position = Point { x: 0., z: 0. };
                f.boss.position = Point { x: 0., z: -12. };
                f.vehicles[owner].loaded = true;
                f.vehicles[owner].pyrite = 40.;
                let target = Point { x: 0., z: -12. };
                f.crates.push(Crate {
                    id: 900,
                    position: target,
                    air: 0.,
                    life: 60.,
                });
                if role == Role::Moto && (slot == 3 || slot == 4) {
                    let carried = slot == 4;
                    f.riders.push(Rider {
                        id: 901,
                        home: 0,
                        position: Point { x: 0., z: 0. },
                        state: if carried {
                            RiderState::Carried
                        } else {
                            RiderState::Ground
                        },
                        hp: 30.,
                    });
                    if carried {
                        f.vehicles[owner].cargo = Some(901);
                    }
                }
                assert!(
                    f.cast(slot, if spec.aimed { Some(target) } else { None })
                        .is_ok(),
                    "{role:?} slot {slot}"
                );
                assert!(f.vehicles[owner].cooldowns[slot] > 0.);
            }
        }
    }
    #[test]
    fn collect_consumes_one_crate_not_the_whole_pile() {
        let mut f = playing(Role::DemoPassenger);
        let p = f.player().body.position;
        f.vehicles[0].pyrite = 20.;
        for id in [900, 901] {
            f.crates.push(Crate {
                id,
                position: p,
                air: 0.,
                life: 60.,
            });
        }
        f.cast(2, Some(p)).unwrap();
        assert_eq!(f.crates.len(), 1);
        assert_eq!(f.player().pyrite, 45.);
    }
    #[test]
    fn insufficient_energy_does_not_trigger_cooldown() {
        let mut f = playing(Role::SiegeDriver);
        f.vehicles[2].energy = 29.;
        assert!(f.cast(1, None).is_err());
        assert_eq!(f.player().cooldowns[1], 0.);
        assert_eq!(f.player().energy, 29.);
    }
    #[test]
    fn two_turret_passengers_trigger_shutdown_and_land() {
        let mut f = playing(Role::DemoDriver);
        f.vehicles[0].hp = 1000.;
        for home in [0, 1] {
            f.vehicles[home].passenger = false;
            f.riders.push(Rider {
                id: 900 + home as u64,
                home,
                position: f.boss.position,
                state: RiderState::Turret,
                hp: 70.,
            });
        }
        advance(&mut f, 9.);
        assert_eq!(f.stats.shutdowns, 1);
        assert!(f.boss.shutdown > 0.);
        assert!(f.riders.iter().all(|r| r.state != RiderState::Turret));
    }
    #[test]
    fn missile_misses_do_not_apply_pyrite_or_damage() {
        let mut f = playing(Role::DemoDriver);
        f.fire(0, Point { x: 45., z: 45. }, ShotKind::Pyrite, true);
        let s = f.shots.pop().unwrap();
        f.impact(s);
        assert_eq!(f.boss.hp, 1000.);
        assert_eq!(f.player().stacks, 0);
        assert_eq!(f.stats.misses, 1);
    }
    #[test]
    fn pursued_target_never_selects_a_motorcycle() {
        let mut f = playing(Role::Moto);
        f.tick(0.02);
        assert!(f.boss.target < 4);
    }
    #[test]
    fn allied_drivers_escape_all_four_corners() {
        for index in [1, 3, 5] {
            for x in [-51_f32, 51.] {
                for z in [-51_f32, 51.] {
                    let mut f = playing(Role::DemoDriver);
                    f.vehicles[index].body.position = Point { x, z };
                    f.boss.position = Point {
                        x: x.signum() * 48.,
                        z: z.signum() * 48.,
                    };
                    for _ in 0..200 {
                        f.ai(index, 0.02);
                    }
                    let p = f.vehicles[index].body.position;
                    assert!(distance(p, Point { x, z }) > 8., "{index} stuck at {x}/{z}");
                    assert!(p.x.abs() <= 51. && p.z.abs() <= 51.);
                }
            }
        }
    }
    #[test]
    fn motorcycle_rescue_never_moves_twice_per_tick() {
        let mut f = playing(Role::DemoDriver);
        let i = 5;
        let original = f.vehicles[i].body.position;
        f.riders.push(Rider {
            id: 900,
            home: 1,
            position: Point {
                x: original.x - 25.,
                z: original.z,
            },
            state: RiderState::Ground,
            hp: 70.,
        });
        f.ai(i, 0.1);
        assert!(distance(original, f.vehicles[i].body.position) <= 1.401);
    }
    fn playing(role: Role) -> Fight {
        let mut f = Fight::new(role, false);
        f.start();
        f
    }
    fn advance(f: &mut Fight, t: f32) {
        for _ in 0..(t * 60.) as usize {
            f.tick(1. / 60.);
        }
    }
    #[test]
    fn five_roles_have_distinct_spell_bars() {
        for r in Role::ALL {
            assert!(spells(r).len() >= 3);
        }
        assert!(!Role::DemoPassenger.drives());
    }
    #[test]
    fn aim_is_required_no_target_lock() {
        let mut f = playing(Role::DemoDriver);
        assert!(f.cast(0, None).is_err());
        assert_eq!(f.stats.shots, 0);
    }
    #[test]
    fn out_of_range_does_not_consume_pyrite() {
        let mut f = playing(Role::DemoDriver);
        let p = f.player().body.position;
        assert!(f.cast(1, Some(p)).is_err());
        assert_eq!(f.player().pyrite, 100.);
    }
    #[test]
    fn projectile_does_not_hit_instantly() {
        let mut f = playing(Role::DemoDriver);
        f.cast(1, Some(f.boss.position)).unwrap();
        assert_eq!(f.player().stacks, 0);
        assert_eq!(f.boss.hp, 1000.);
        assert_eq!(f.player().pyrite, 95.);
    }
    #[test]
    fn stationary_impact_applies_pyrite() {
        let mut f = playing(Role::DemoDriver);
        let p = f.boss.position;
        f.fire(0, p, ShotKind::Pyrite, true);
        let s = f.shots.pop().unwrap();
        f.impact(s);
        assert_eq!(f.vehicles[0].stacks, 1);
        assert_eq!(f.stats.hits, 1);
    }
    #[test]
    fn moving_boss_can_dodge_a_previously_aimed_shot() {
        let mut f = playing(Role::DemoDriver);
        f.fire(0, f.boss.position, ShotKind::Pyrite, true);
        let shot = f.shots.pop().unwrap();
        f.boss.position.x += 20.;
        f.impact(shot);
        assert_eq!(f.stats.misses, 1);
        assert_eq!(f.vehicles[0].stacks, 0);
    }
    #[test]
    fn cooldown_blocks_repeat() {
        let mut f = playing(Role::DemoDriver);
        let p = f.boss.position;
        f.cast(0, Some(p)).unwrap();
        assert!(f.cast(0, Some(p)).is_err());
    }
    #[test]
    fn electroshock_requires_front_and_energy() {
        let mut f = playing(Role::SiegeDriver);
        f.vehicles[2].body.position = Point { x: 0., z: 0. };
        f.boss.position = Point { x: 0., z: -10. };
        f.boss.cast = 2.;
        f.cast(1, None).unwrap();
        assert_eq!(f.boss.cast, 0.);
        assert_eq!(f.stats.interrupts, 1);
        assert_eq!(f.player().energy, 70.);
    }
    #[test]
    fn electroshock_cannot_interrupt_behind() {
        let mut f = playing(Role::SiegeDriver);
        f.vehicles[2].body.position = Point { x: 0., z: 0. };
        f.boss.position = Point { x: 0., z: 10. };
        f.boss.cast = 2.;
        f.cast(1, None).unwrap();
        assert_eq!(f.boss.cast, 2.);
    }
    #[test]
    fn gunner_has_independent_resource() {
        let mut f = playing(Role::SiegeGunner);
        f.cast(1, Some(f.boss.position)).unwrap();
        assert_eq!(f.player().gun_energy, 80.);
        assert_eq!(f.player().energy, 100.);
    }
    #[test]
    fn collecting_requires_ground_crate() {
        let mut f = playing(Role::DemoPassenger);
        let p = f.player().body.position;
        f.vehicles[0].pyrite = 30.;
        f.crates.push(Crate {
            id: 99,
            position: p,
            air: 3.,
            life: 50.,
        });
        assert!(f.cast(2, Some(p)).is_err());
        f.crates[0].air = 0.;
        f.cast(2, Some(p)).unwrap();
        assert_eq!(f.player().pyrite, 55.);
    }
    #[test]
    fn anti_air_drops_supply_not_boss_damage() {
        let mut f = playing(Role::DemoPassenger);
        let p = f.boss.position;
        f.crates.push(Crate {
            id: 99,
            position: p,
            air: 3.,
            life: 50.,
        });
        f.fire(0, p, ShotKind::Rocket, true);
        let shot = f.shots.pop().unwrap();
        f.impact(shot);
        assert_eq!(f.crates[0].air, 0.);
        assert_eq!(f.boss.hp, 1000.);
    }
    #[test]
    fn projection_requires_loaded_passenger() {
        let mut f = playing(Role::DemoDriver);
        assert!(f.cast(3, Some(f.boss.position)).is_err());
        f.vehicles[0].loaded = true;
        f.cast(3, Some(f.boss.position)).unwrap();
        assert!(!f.player().passenger);
    }
    #[test]
    fn motorcycle_pickup_return_and_heal() {
        let mut f = playing(Role::Moto);
        let p = f.player().body.position;
        f.riders.push(Rider {
            id: 99,
            home: 0,
            position: p,
            state: RiderState::Ground,
            hp: 30.,
        });
        f.vehicles[0].passenger = false;
        f.cast(3, None).unwrap();
        assert_eq!(f.player().cargo, Some(99));
        f.cast(4, None).unwrap();
        assert_eq!(f.riders[0].hp, 100.);
        f.vehicles[4].body.position = f.vehicles[0].body.position;
        f.vehicles[4].cooldowns[3] = 0.;
        f.cast(3, None).unwrap();
        assert!(f.vehicles[0].passenger);
        assert_eq!(f.stats.rescued, 1);
    }
    #[test]
    fn ready_does_not_advance() {
        let mut f = Fight::new(Role::Moto, true);
        f.tick(0.1);
        assert_eq!(f.time, 0.);
    }
    #[test]
    fn survival_simulation_has_finite_bounded_positions() {
        let mut f = playing(Role::DemoDriver);
        f.vehicles[0].hp = 10000.;
        advance(&mut f, 40.);
        assert!(f.vehicles.iter().all(|v| v.body.position.x.is_finite()
            && v.body.position.x.abs() <= 54.
            && v.body.position.z.abs() <= 54.));
        assert!(f.boss.hp < 1000.);
    }
}
