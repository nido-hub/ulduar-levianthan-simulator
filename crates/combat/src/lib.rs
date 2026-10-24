//! Headless Flame Leviathan encounter. Reference and training exceptions: docs/wotlk-reference.md.
pub mod data;
use leviathan_driving::{Point, Vehicle};
use std::f32::consts::TAU;

pub fn distance(a: Point, b: Point) -> f32 {
    (a.x - b.x).hypot(a.z - b.z)
}
fn bounded(p: Point) -> Point {
    Point {
        x: p.x.clamp(-data::EDGE, data::EDGE),
        z: p.z.clamp(-data::EDGE, data::EDGE),
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum School {
    Physical,
    Fire,
    Frost,
    Nature,
    Arcane,
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
    pub gcd: f32,
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
        gcd: 1.,
    }
}
pub fn spells(role: Role) -> Vec<Spell> {
    let mut bar = match role {
        Role::DemoDriver => vec![
            spell("Rocher", true, 10., 70., 0., 0., 20.),
            spell("Pyrite", true, 10., 70., 5., 0., 20.),
            spell("Bélier", false, 0., 18., 0., 4., 18.),
            spell("Projeter passager", true, 10., 70., 0., 2., 10.),
        ],
        Role::DemoPassenger => vec![
            spell("Mortier", true, 0., 50., 0., 1., 11.),
            spell("Roquette anti-air", true, 0., 100., 0., 0.25, 3.),
            spell("Saisir une caisse", true, 0., 40., 0., 0., 3.),
            spell("Accélération", false, 0., 0., 25., 0., 0.),
            spell("Charger catapulte", false, 0., 0., 0., 30., 0.),
        ],
        Role::SiegeDriver => vec![
            spell("Bélier", false, 0., 18., 40., 0., 18.),
            spell("Électrochoc", false, 0., 25., 38., 10., 25.),
            spell("Ruée vapeur", false, 0., 0., 40., 15., 35.),
        ],
        Role::SiegeGunner => vec![
            spell("Roquette anti-air", true, 0., 100., 10., 0.25, 3.),
            spell("Canon", true, 10., 70., 20., 0., 20.),
            spell("Bouclier", false, 0., 0., 0., 60., 0.),
        ],
        Role::Moto => vec![
            spell("Cor sonore", false, 0., 35., 20., 0., 35.),
            spell("Goudron", false, 0., 0., 0., 15., 9.),
            spell("Accélération", false, 0., 0., 50., 0., 0.),
            spell("Secours / déposer", false, 0., 7., 0., 0.5, 0.),
            spell("Premiers soins", false, 0., 0., 0., 60., 0.),
        ],
    };
    match role {
        Role::DemoDriver => bar[2].gcd = 1.5,
        Role::DemoPassenger => {
            bar[0].gcd = 0.;
            bar[1].gcd = 0.;
        }
        Role::SiegeDriver => {
            for s in &mut bar {
                s.gcd = 1.5;
            }
            bar[2].gcd = 2.;
        }
        Role::SiegeGunner => {
            bar[0].gcd = 0.;
            bar[2].gcd = 1.5;
        }
        Role::Moto => {
            bar[3].gcd = 0.;
            bar[4].gcd = 0.;
        }
    }
    bar
}
#[derive(Clone, Debug)]
pub struct CrewVehicle {
    pub kind: Kind,
    pub body: Vehicle,
    pub hp: f32,
    pub max_hp: f32,
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
    pub gcd: [f32; 2],
    pub vulnerability: u8,
    pub vulnerability_time: f32,
    pub napalm: f32,
    pub healing: f32,
    pub dot_clock: f32,
    pub passenger_hp: f32,
}
impl CrewVehicle {
    pub fn speed_multiplier(&self) -> f32 {
        if self.boost > 0. { 2. } else { 1. }
    }
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
fn shot_damage(kind: ShotKind) -> (f32, School) {
    match kind {
        ShotKind::Boulder => (31_000., School::Fire),
        ShotKind::Pyrite => (60_000., School::Arcane),
        ShotKind::Mortar => (32_000., School::Fire),
        ShotKind::Cannon => (80_000., School::Physical),
        _ => (0., School::Physical),
    }
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
impl Hazard {
    pub fn strike_at(&self) -> f32 {
        match self.kind {
            HazardKind::Storm => self.duration - 5.,
            HazardKind::Frost => 5.,
            HazardKind::Fire => 0.8,
            HazardKind::Tar => 0.,
        }
    }
    pub fn striking(&self) -> bool {
        self.age >= self.strike_at()
    }
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
    pub ward: bool,
    pub attack: f32,
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
    pub max_hp: f32,
    pub target: usize,
    pub pursuit: f32,
    pub cast: f32,
    pub vents: f32,
    pub shutdown: f32,
    pub turret_timer: f32,
    pub missile: f32,
    pub travelled: f32,
    pub speed_stacks: u8,
    pub turrets: [f32; 2],
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
    speed_timer: f32,
    ram_timer: f32,
    vents_tick: f32,
    napalm_timer: f32,
    napalm_cast: f32,
    napalm_target: usize,
    turret_attack: bool,
    turret_clock: f32,
    eject_timer: f32,
    fire_position: Point,
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
            hp: data::vehicle_hp(kind),
            max_hp: data::vehicle_hp(kind),
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
            gcd: [0.; 2],
            vulnerability: 0,
            vulnerability_time: 0.,
            napalm: 0.,
            healing: 0.,
            dot_clock: 0.,
            passenger_hp: data::RIDER_HP,
        })
        .collect();
        Self {
            role,
            vehicles,
            boss: Boss {
                position: Point { x: 0., z: -17. },
                hp: data::boss_hp(towers),
                max_hp: data::boss_hp(towers),
                target: 2,
                pursuit: 0.,
                cast: 0.,
                vents: 20.,
                shutdown: 0.,
                turret_timer: 0.,
                missile: 5.,
                travelled: 0.,
                speed_stacks: 0,
                turrets: [data::TURRET_HP; 2],
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
            storm_timer: 60.,
            fire_timer: 42.,
            fire_phase: 0,
            frost_timer: 20.,
            life_timer: 34.,
            launch_timer: 48.,
            speed_timer: 15.,
            ram_timer: 0.,
            vents_tick: 0.,
            napalm_timer: 12.,
            napalm_cast: 0.,
            napalm_target: 0,
            turret_attack: false,
            turret_clock: 0.,
            eject_timer: 0.,
            fire_position: Point {
                x: data::FIRE_PATH[0].0,
                z: data::FIRE_PATH[0].1,
            },
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
            self.boss.max_hp = data::boss_hp(self.towers);
            self.boss.hp = self.boss.max_hp;
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
    pub fn set_towers(&mut self, towers: bool) {
        if self.outcome == Outcome::Ready {
            self.towers = towers;
            self.boss.max_hp = data::boss_hp(towers);
            self.boss.hp = self.boss.max_hp;
        }
    }
    pub fn cooldowns(&self) -> [f32; 5] {
        let mut remaining = if self.role.drives() {
            self.player().cooldowns
        } else {
            self.player().ai_cd
        };
        let gcd = self.player().gcd[usize::from(!self.role.drives())];
        for (slot, spell) in spells(self.role).iter().enumerate() {
            if spell.gcd > 0. {
                remaining[slot] = remaining[slot].max(gcd);
            }
        }
        remaining
    }
    pub fn onboard(&self) -> bool {
        self.role != Role::DemoPassenger || self.player().passenger
    }
    pub fn player_rider(&self) -> Option<&Rider> {
        if self.role != Role::DemoPassenger {
            return None;
        }
        self.riders.iter().find(|r| r.home == self.role.index())
    }
    pub fn player_position(&self) -> Point {
        if let Some(r) = self.player_rider() {
            return r.position;
        }
        if self.role == Role::DemoPassenger
            && let Some(s) = self
                .shots
                .iter()
                .find(|s| s.owner == self.role.index() && s.kind == ShotKind::Passenger)
        {
            let t = (s.age / s.flight).clamp(0., 1.);
            return Point {
                x: s.start.x + (s.end.x - s.start.x) * t,
                z: s.start.z + (s.end.z - s.start.z) * t,
            };
        }
        self.player().body.position
    }
    pub fn health_fraction(&self) -> f32 {
        if self.role == Role::DemoPassenger
            && !self.player().passenger
            && self
                .shots
                .iter()
                .any(|s| s.owner == self.role.index() && s.kind == ShotKind::Passenger)
        {
            return (self.player().passenger_hp / data::RIDER_HP).clamp(0., 1.);
        }
        self.player_rider()
            .map_or(self.player().hp / self.player().max_hp, |r| {
                r.hp / data::RIDER_HP
            })
            .clamp(0., 1.)
    }
    pub fn active_spells(&self) -> Vec<Spell> {
        if let Some(r) = self.player_rider() {
            return if r.state == RiderState::Turret && r.hp > 0. {
                vec![spell("Attaquer les tourelles", false, 0., 0., 0., 0., 0.)]
            } else {
                vec![]
            };
        }
        if self.role == Role::DemoPassenger && (!self.onboard() || self.player().loaded) {
            return vec![];
        }
        spells(self.role)
    }
    pub fn attacking_turrets(&self) -> bool {
        self.turret_attack
    }
    pub fn player_can_move(&self) -> bool {
        if !matches!(self.outcome, Outcome::Ready | Outcome::Fighting) {
            return false;
        }
        if let Some(r) = self.player_rider() {
            return r.state == RiderState::Ground && r.hp > 0.;
        }
        self.role.drives() && self.onboard() && self.player().hp > 0. && self.player().frozen <= 0.
    }
    pub fn aim_valid(&self, index: usize, point: Point) -> bool {
        let Some(spell) = spells(self.role).get(index).copied() else {
            return false;
        };
        let d = distance(self.player().body.position, point);
        point.x.is_finite()
            && point.z.is_finite()
            && point.x.abs() <= data::ARENA_HALF
            && point.z.abs() <= data::ARENA_HALF
            && d >= spell.min
            && d <= spell.max
    }
    pub fn cast(&mut self, index: usize, aim: Option<Point>) -> Result<(), &'static str> {
        if let Some(r) = self.player_rider() {
            if self.outcome == Outcome::Fighting
                && r.state == RiderState::Turret
                && r.hp > 0.
                && index == 0
                && aim.is_none()
            {
                self.turret_attack = true;
                self.say("Attaque des tourelles : surcharge après leur destruction.");
                return Ok(());
            }
            return Err("Passager hors véhicule : aucune compétence de véhicule disponible.");
        }
        self.cast_for(self.role.index(), self.role, index, aim, true)
    }
    fn cast_for(
        &mut self,
        owner: usize,
        role: Role,
        index: usize,
        aim: Option<Point>,
        personal: bool,
    ) -> Result<(), &'static str> {
        if self.outcome != Outcome::Fighting {
            return Err("Lance le combat avec Entrée.");
        }
        let Some(spell) = spells(role).get(index).copied() else {
            return Err("Pas de sort sur cette touche.");
        };
        let v = &self.vehicles[owner];
        if role == Role::DemoPassenger && (!v.passenger || v.loaded) {
            return Err("Passager hors véhicule : retour par une moto en cours.");
        }
        if v.hp <= 0. || v.frozen > 0. {
            return Err("Véhicule détruit ou gelé.");
        }
        let cds = if role.drives() {
            &v.cooldowns
        } else {
            &v.ai_cd
        };
        let seat = usize::from(!role.drives());
        if cds[index] > 0. || (spell.gcd > 0. && v.gcd[seat] > 0.) {
            return Err("Sort en recharge.");
        }
        let resource = match role {
            Role::DemoDriver | Role::DemoPassenger => v.pyrite,
            Role::SiegeGunner => v.gun_energy,
            _ => v.energy,
        };
        if resource < spell.cost {
            return Err("Ressource insuffisante.");
        }
        if spell.aimed
            && !aim.is_some_and(|p| {
                p.x.is_finite()
                    && p.z.is_finite()
                    && p.x.abs() <= data::ARENA_HALF
                    && p.z.abs() <= data::ARENA_HALF
                    && distance(v.body.position, p) >= spell.min
                    && distance(v.body.position, p) <= spell.max
            })
        {
            return Err("Visée hors portée ou hors de la cour.");
        }
        // Preconditions must be checked before consuming any resource or cooldown.
        match (role, index) {
            (Role::DemoPassenger, 3) if v.boost > 0. => return Err("Accélération déjà active."),
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
                    if self.vehicles[r.home].hp > 0.
                        && distance(v.body.position, self.vehicles[r.home].body.position) > 8.
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
        if role.drives() {
            v.cooldowns[index] = spell.cooldown;
        } else {
            v.ai_cd[index] = spell.cooldown;
        }
        if spell.gcd > 0. {
            v.gcd[seat] = spell.gcd;
        }
        match role {
            Role::DemoDriver | Role::DemoPassenger => v.pyrite -= spell.cost,
            Role::SiegeGunner => v.gun_energy -= spell.cost,
            _ => v.energy -= spell.cost,
        }
        match (role, index) {
            (Role::DemoDriver, 0) => self.fire(owner, aim.unwrap(), ShotKind::Boulder, personal),
            (Role::DemoDriver, 1) => self.fire(owner, aim.unwrap(), ShotKind::Pyrite, personal),
            (Role::DemoDriver, 2) => self.frontal(owner, 18., 20_000.),
            (Role::SiegeDriver, 0) => self.frontal(owner, 18., 25_001.),
            (Role::DemoDriver, 3) => {
                self.vehicles[owner].passenger = false;
                self.vehicles[owner].loaded = false;
                self.fire(owner, aim.unwrap(), ShotKind::Passenger, personal);
            }
            (Role::DemoPassenger, 0) => self.fire(owner, aim.unwrap(), ShotKind::Mortar, personal),
            (Role::DemoPassenger, 1) | (Role::SiegeGunner, 0) => {
                self.fire(owner, aim.unwrap(), ShotKind::Rocket, personal)
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
            (Role::DemoPassenger, 3) => self.vehicles[owner].boost = 20.,
            (Role::SiegeDriver, 2) | (Role::Moto, 2) => self.vehicles[owner].boost = 5.,
            (Role::DemoPassenger, 4) => {
                self.vehicles[owner].loaded = true;
                self.say("Chargé : le conducteur IA prépare ta projection.");
            }
            (Role::SiegeDriver, 1) => {
                if self.in_front(owner, self.boss.position, 25.) && self.boss.cast > 0. {
                    self.boss.cast = 0.;
                    if personal {
                        self.stats.interrupts += 1;
                    }
                    self.say("Électrochoc : interruption réussie.");
                } else {
                    self.say("Électrochoc : pas d'incantation à portée dans le cône.");
                }
                self.frontal_school(owner, 25., 8_720., School::Nature);
            }
            (Role::SiegeGunner, 1) => self.fire(owner, aim.unwrap(), ShotKind::Cannon, personal),
            (Role::SiegeGunner, 2) => self.vehicles[owner].shield = 5.,
            (Role::Moto, 0) => self.frontal(owner, 35., 7_000.),
            (Role::Moto, 1) => {
                let position = self.vehicles[owner].body.position;
                let id = self.id();
                self.hazards.push(Hazard {
                    id,
                    kind: HazardKind::Tar,
                    position,
                    radius: 9.,
                    age: 0.,
                    duration: 45.,
                    burning: false,
                    target: owner,
                });
            }
            (Role::Moto, 3) => {
                self.rescue(owner);
            }
            (Role::Moto, 4) => {
                self.vehicles[owner].healing = 4.;
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
        self.frontal_school(owner, range, damage, School::Physical);
    }
    fn frontal_school(&mut self, owner: usize, range: f32, damage: f32, school: School) {
        let damage = damage * data::GEAR_SCALE;
        if self.in_front(owner, self.boss.position, range) {
            self.damage_boss_school(damage, school);
        }
        let hits: Vec<_> = self
            .adds
            .iter()
            .filter(|a| self.in_front(owner, a.position, range))
            .map(|a| a.id)
            .collect();
        for a in &mut self.adds {
            if hits.contains(&a.id) {
                a.hp -= damage;
            }
        }
    }
    fn damage_boss(&mut self, damage: f32) {
        self.damage_boss_school(damage, School::Arcane);
    }
    fn damage_boss_school(&mut self, damage: f32, school: School) {
        self.boss.hp = (self.boss.hp
            - damage
                * if self.boss.shutdown > 0. { 1.5 } else { 1. }
                * if self.towers && school == School::Fire {
                    0.9
                } else {
                    1.
                })
        .max(0.);
    }
    fn harm(&mut self, index: usize, damage: f32, avoidable: bool) {
        self.harm_school(index, damage, avoidable, School::Physical, true);
    }
    fn harm_school(
        &mut self,
        index: usize,
        damage: f32,
        avoidable: bool,
        school: School,
        boss_source: bool,
    ) {
        let v = &mut self.vehicles[index];
        let damage = damage
            * (1. + v.vulnerability as f32 * 0.5)
            * if v.shield > 0. && school != School::Nature {
                0.85
            } else {
                1.
            }
            * if self.towers && boss_source {
                match school {
                    School::Physical => 1.25,
                    School::Fire => 1.5,
                    _ => 1.,
                }
            } else {
                1.
            };
        let actual = damage.min(v.hp);
        v.hp = (v.hp - damage).max(0.);
        if index == self.role.index() && avoidable {
            self.stats.avoidable += actual;
        }
    }
    fn impact(&mut self, shot: Shot) {
        let radius = match shot.kind {
            ShotKind::Pyrite => 20.,
            ShotKind::Rocket => 3.,
            ShotKind::Mortar => 11.,
            ShotKind::Passenger => 10.,
            _ => 20.,
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
                hp: self.vehicles[shot.owner].passenger_hp,
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
                let (raw, school) = shot_damage(shot.kind);
                self.damage_boss_school(raw * data::GEAR_SCALE, school);
                if shot.kind == ShotKind::Pyrite {
                    let v = &mut self.vehicles[shot.owner];
                    v.stacks = (v.stacks + 1).min(10);
                    v.stack_time = 10.;
                }
            }
            for a in &mut self.adds {
                if distance(a.position, shot.end) < radius + 1. {
                    a.hp -= shot_damage(shot.kind).0 * data::GEAR_SCALE;
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
            if r.hp <= 0. || r.state == RiderState::Lost {
                self.vehicles[moto].cargo = None;
                return false;
            }
            let home = r.home;
            if self.vehicles[home].hp <= 0. {
                self.vehicles[moto].cargo = None;
                if let Some(r) = self.riders.iter_mut().find(|r| r.id == id) {
                    r.state = RiderState::Ground;
                    r.position = position;
                }
                self.say("Démolisseur détruit : passager déposé au sol.");
                return true;
            }
            if distance(position, self.vehicles[home].body.position) < 8. {
                self.vehicles[home].passenger = true;
                self.vehicles[home].passenger_hp = r.hp;
                self.vehicles[moto].cargo = None;
                self.riders.retain(|r| r.id != id);
                if moto == self.role.index() {
                    self.stats.rescued += 1;
                }
                self.say("Passager ramené à son démolisseur.");
                return true;
            }
        } else if let Some(r) = self.riders.iter_mut().find(|r| {
            r.state == RiderState::Ground && r.hp > 0. && distance(r.position, position) < 7.
        }) {
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
        if v.kind == Kind::Demo
            && (!is_player || self.role != Role::DemoPassenger)
            && v.passenger
            && !v.loaded
            && self.boss.target == index
            && self.boss.shutdown <= 0.
            && v.boost <= 0.
        {
            let _ = self.cast_for(index, Role::DemoPassenger, 3, None, false);
        }
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
                        .filter(|r| {
                            r.state == RiderState::Ground
                                && r.hp > 0.
                                && self.vehicles[r.home].hp > 0.
                                && !reserved.contains(&r.id)
                        })
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
                if v.kind == Kind::Siege
                    && distance(v.body.position, self.boss.position) < 30.
                    && v.boost <= 0.
                {
                    let _ = self.cast_for(index, Role::SiegeDriver, 2, None, false);
                }
            } else {
                let radius = if v.kind == Kind::Demo { 32. } else { 14. };
                let base = (v.body.position.z - self.boss.position.z)
                    .atan2(v.body.position.x - self.boss.position.x);
                goal = v.body.position;
                let mut best = f32::INFINITY;
                for step in 0..24 {
                    let a = base + step as f32 * TAU / 24.;
                    let p = Point {
                        x: self.boss.position.x + radius * a.cos(),
                        z: self.boss.position.z + radius * a.sin(),
                    };
                    if p.x.abs() > data::EDGE || p.z.abs() > data::EDGE {
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
            let speed = data::BASE_SPEED * self.vehicles[index].speed_multiplier();
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
                if v.cargo.is_some()
                    || self.riders.iter().any(|r| {
                        r.state == RiderState::Ground
                            && r.hp > 0.
                            && self.vehicles[r.home].hp > 0.
                            && distance(r.position, self.vehicles[index].body.position) < 7.
                    })
                {
                    self.rescue(index);
                }
            }
        }
        let pos = self.vehicles[index].body.position;
        let d = distance(pos, self.boss.position);
        // Driver and passenger automation are independent; never fire the player's spells.
        if v.kind == Kind::Demo {
            let ai_passenger = !is_player || self.role != Role::DemoPassenger;
            if ai_passenger && v.passenger && !v.loaded && v.pyrite <= 75. {
                if let Some(c) = self
                    .crates
                    .iter()
                    .find(|c| c.air <= 0. && distance(c.position, pos) < 40.)
                    .map(|c| c.position)
                {
                    let _ = self.cast_for(index, Role::DemoPassenger, 2, Some(c), false);
                }
            }
            if ai_passenger && v.passenger && !v.loaded && self.launch_timer < 4. {
                let _ = self.cast_for(index, Role::DemoPassenger, 4, None, false);
            }
            if driver
                && v.loaded
                && v.passenger
                && d < 65.
                && self.boss.shutdown <= 0.
                && (self.launch_timer <= 0.1 || (is_player && self.role == Role::DemoPassenger))
            {
                let _ = self.cast_for(index, Role::DemoDriver, 3, Some(self.boss.position), false);
            }
            if driver && d < 70. && d >= 10. {
                let slot = if v.pyrite >= 5. && (v.stacks < 10 || v.stack_time < 3.) {
                    1
                } else {
                    0
                };
                let _ = self.cast_for(
                    index,
                    Role::DemoDriver,
                    slot,
                    Some(self.boss.position),
                    false,
                );
            }
            if ai_passenger && v.passenger && !v.loaded {
                let airborne = self
                    .crates
                    .iter()
                    .find(|c| c.air > 0. && distance(c.position, pos) < 100.)
                    .map(|c| c.position);
                if let Some(point) = airborne {
                    let _ = self.cast_for(index, Role::DemoPassenger, 1, Some(point), false);
                }
                let target = self
                    .vehicles
                    .iter()
                    .filter(|a| a.hp > 0. && a.frozen > 0. && distance(a.body.position, pos) < 50.)
                    .map(|a| a.body.position)
                    .next()
                    .unwrap_or(self.boss.position);
                if distance(pos, target) < 50. {
                    let _ = self.cast_for(index, Role::DemoPassenger, 0, Some(target), false);
                }
            }
        } else if v.kind == Kind::Siege {
            if driver
                && self.boss.cast > 0.
                && d < 25.
                && self.in_front(index, self.boss.position, 25.)
            {
                let _ = self.cast_for(index, Role::SiegeDriver, 1, None, false);
            }
            if !is_player || self.role != Role::SiegeGunner {
                let _ = self.cast_for(index, Role::SiegeGunner, 1, Some(self.boss.position), false);
            }
        } else if driver {
            if distance(pos, self.boss.position) < 24. {
                let _ = self.cast_for(index, Role::Moto, 1, None, false);
            }
            if self
                .adds
                .iter()
                .any(|a| self.in_front(index, a.position, 35.))
            {
                let _ = self.cast_for(index, Role::Moto, 0, None, false);
            }
            if v.cargo.is_some() {
                let _ = self.cast_for(index, Role::Moto, 4, None, false);
            }
        }
    }
    fn release_destroyed_carriers(&mut self) {
        for v in &mut self.vehicles {
            if v.hp <= 0.
                && let Some(id) = v.cargo.take()
                && let Some(r) = self.riders.iter_mut().find(|r| r.id == id)
            {
                r.position = v.body.position;
                r.state = if r.hp > 0. {
                    RiderState::Ground
                } else {
                    RiderState::Lost
                };
            }
        }
    }
    fn harm_exposed_riders(&mut self, dt: f32, vents_pulse: bool) {
        for r in &mut self.riders {
            if !matches!(r.state, RiderState::Ground | RiderState::Carried) || r.hp <= 0. {
                continue;
            }
            let mut damage = 0.;
            if vents_pulse && distance(r.position, self.boss.position) <= 50. {
                damage += 3_000. * if self.towers { 1.5 } else { 1. };
            }
            if self.hazards.iter().any(|h| {
                h.kind == HazardKind::Fire
                    && h.striking()
                    && distance(h.position, r.position) <= h.radius
            }) {
                damage += data::RIDER_HP * 0.10 * dt;
            }
            for h in &self.hazards {
                if h.kind == HazardKind::Tar || h.age < h.strike_at() || h.age - dt >= h.strike_at()
                {
                    continue;
                }
                let d = distance(h.position, r.position);
                match h.kind {
                    HazardKind::Storm => {
                        damage += if d <= 7. {
                            300_000.
                        } else {
                            300_000. / (d - 6.).max(1.)
                        }
                    }
                    HazardKind::Fire | HazardKind::Frost if d <= h.radius => damage += 150_000.,
                    _ => {}
                }
            }
            r.hp = (r.hp - damage).max(0.);
            if r.hp == 0. {
                r.state = RiderState::Lost;
            }
        }
    }
    pub fn tick(&mut self, dt: f32) {
        if self.outcome != Outcome::Fighting || !dt.is_finite() || dt <= 0. || dt > 0.1 {
            return;
        }
        self.time += dt;
        self.release_destroyed_carriers();
        let mut vents_pulse = false;
        for v in &mut self.vehicles {
            for cd in v
                .cooldowns
                .iter_mut()
                .chain(v.ai_cd.iter_mut())
                .chain(v.gcd.iter_mut())
            {
                *cd = (*cd - dt).max(0.);
            }
            v.energy = (v.energy + 10. * dt).min(100.);
            v.gun_energy = (v.gun_energy + 10. * dt).min(100.);
            for timer in [&mut v.boost, &mut v.shield, &mut v.frozen] {
                *timer = (*timer - dt).max(0.);
            }
            v.vulnerability_time = (v.vulnerability_time - dt).max(0.);
            if v.vulnerability_time == 0. {
                v.vulnerability = 0;
            }
        }
        for i in 0..6 {
            self.ai(i, dt);
        }
        for i in 0..6 {
            let stacks = self.vehicles[i].stacks;
            if stacks > 0 {
                self.damage_boss(stacks as f32 * data::PYRITE_TICK * data::GEAR_SCALE * dt);
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
                self.boss.turrets = [data::TURRET_HP; 2];
                self.speed_timer = 15.;
            }
        } else {
            self.speed_timer -= dt;
            if self.speed_timer <= 0. {
                self.speed_timer += 15.;
                self.boss.speed_stacks = (self.boss.speed_stacks + 1).min(20);
            }
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
                if self.boss.cast > 0. {
                    0.
                } else {
                    5. * (1. + self.boss.speed_stacks as f32 * 0.05)
                        * dt
                        * if slow { 0.5 } else { 1. }
                },
            );
            self.boss.travelled += distance(old_boss, self.boss.position);
            self.ram_timer = (self.ram_timer - dt).max(0.);
            if self.ram_timer == 0.
                && self.boss.cast <= 0.
                && distance(target, self.boss.position) < 15.
            {
                self.ram_timer = 2.;
                self.harm(self.boss.target, 10_261., true);
                let v = &mut self.vehicles[self.boss.target];
                v.vulnerability = v.vulnerability.saturating_add(1);
                v.vulnerability_time = 20.;
            }
            self.boss.vents -= dt;
            if self.boss.vents <= 0. {
                self.boss.vents += 20.;
                self.boss.cast = data::VENTS_DURATION;
                self.vents_tick = 0.;
                self.say("Décharges de flammes : interrompre !");
            }
            if self.boss.cast > 0. {
                self.vents_tick -= dt;
                self.boss.cast = (self.boss.cast - dt).max(0.);
                if self.vents_tick <= 0. {
                    vents_pulse = true;
                    self.vents_tick += 1.;
                    self.stats.vents_missed += 1;
                    for i in 0..6 {
                        if distance(self.vehicles[i].body.position, self.boss.position) < 50. {
                            self.harm_school(i, 3_000., true, School::Fire, true);
                        }
                    }
                }
            }
            self.boss.missile -= dt;
            if self.boss.missile <= 0. {
                self.boss.missile += 4.;
                let alive: Vec<_> = (0..6).filter(|i| self.vehicles[*i].hp > 0.).collect();
                if !alive.is_empty() {
                    self.harm(alive[(self.time as usize / 4) % alive.len()], 4_000., false);
                }
            }
            self.napalm_timer -= dt;
            if self.napalm_timer <= 0. && self.napalm_cast <= 0. {
                self.napalm_timer += 12.;
                self.napalm_target = (self.time as usize / 12) % 6;
                self.napalm_cast = 2.;
            }
            if self.napalm_cast > 0. {
                self.napalm_cast -= dt;
                if self.napalm_cast <= 0. {
                    let p = self.vehicles[self.napalm_target].body.position;
                    for i in 0..6 {
                        if distance(self.vehicles[i].body.position, p) <= 5. {
                            self.harm_school(i, 8_000., true, School::Fire, true);
                            self.vehicles[i].napalm = 8.;
                        }
                    }
                }
            }
        }
        for i in 0..6 {
            self.vehicles[i].dot_clock -= dt;
            if self.vehicles[i].dot_clock <= 0. {
                self.vehicles[i].dot_clock += 1.;
                if self.vehicles[i].frozen > 0. {
                    self.harm_school(
                        i,
                        self.vehicles[i].max_hp * 0.01,
                        true,
                        School::Frost,
                        false,
                    );
                }
                if self.vehicles[i].napalm > 0. {
                    self.harm_school(i, 4_000., true, School::Fire, true);
                }
                if self.vehicles[i].healing > 0.
                    && let Some(id) = self.vehicles[i].cargo
                    && let Some(r) = self.riders.iter_mut().find(|r| r.id == id && r.hp > 0.)
                {
                    r.hp = (r.hp + data::RIDER_HP * 0.3).min(data::RIDER_HP);
                }
            }
            self.vehicles[i].napalm = (self.vehicles[i].napalm - dt).max(0.);
            self.vehicles[i].healing = (self.vehicles[i].healing - dt).max(0.);
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
                    air: 180.,
                    life: 180.,
                });
            }
        }
        for c in &mut self.crates {
            c.life -= dt;
        }
        self.crates.retain(|c| c.life > 0.);
        // Non-player gunners shoot down supplies after a short delay, not instant refills.
        self.launch_timer -= dt;
        if self.launch_timer <= 0. {
            self.launch_timer = 60.;
        }
        let attackers = self
            .riders
            .iter()
            .filter(|r| {
                r.state == RiderState::Turret
                    && r.hp > 0.
                    && (self.role != Role::DemoPassenger
                        || r.home != self.role.index()
                        || self.turret_attack)
            })
            .count();
        if self.boss.shutdown <= 0. {
            if let Some(turret) = self.boss.turrets.iter_mut().find(|hp| **hp > 0.) {
                *turret = (*turret - attackers as f32 * data::RIDER_DPS * dt).max(0.);
            }
            let living = self.boss.turrets.iter().filter(|hp| **hp > 0.).count();
            let riders = self
                .riders
                .iter()
                .filter(|r| r.state == RiderState::Turret && r.hp > 0.)
                .count();
            if riders == 0 {
                self.turret_clock = 0.;
            } else {
                self.turret_clock += dt;
            }
            if riders > 0 && self.turret_clock >= 2. {
                self.turret_clock -= 2.;
                if let Some(r) = self
                    .riders
                    .iter_mut()
                    .find(|r| r.state == RiderState::Turret && r.hp > 0.)
                {
                    r.hp = (r.hp - living as f32 * 500.).max(0.);
                    if r.hp == 0. {
                        r.state = RiderState::Lost;
                    }
                }
            }
            if self.boss.turrets.iter().all(|hp| *hp <= 0.) {
                self.boss.shutdown = 20.;
                self.boss.cast = 0.;
                self.napalm_cast = 0.;
                self.boss.speed_stacks = 0;
                self.eject_timer = 3.;
                self.turret_attack = false;
                self.stats.shutdowns += 1;
                self.say("Surcharge : 20 secondes, éjection puis secours !");
            }
        }
        if self.eject_timer > 0. {
            self.eject_timer -= dt;
            if self.eject_timer <= 0. {
                for (i, r) in self.riders.iter_mut().enumerate() {
                    if r.state == RiderState::Turret {
                        r.state = RiderState::Ground;
                        r.position = bounded(Point {
                            x: self.boss.position.x + if i % 2 == 0 { -10. } else { 10. },
                            z: self.boss.position.z + 10.,
                        });
                    }
                }
            }
        }
        for r in &mut self.riders {
            if r.state == RiderState::Turret {
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
            if h.kind == HazardKind::Frost && h.age < 0. {
                h.position = toward(h.position, self.vehicles[h.target].body.position, 2.5 * dt);
                if distance(h.position, self.vehicles[h.target].body.position) <= 0.6 {
                    h.age = 0.;
                }
            } else {
                h.age += dt;
            }
        }
        for i in 0..6 {
            let p = self.vehicles[i].body.position;
            let fire = self.hazards.iter().any(|h| {
                h.kind == HazardKind::Fire && h.age > 0.8 && distance(h.position, p) < h.radius
            });
            if fire {
                self.harm_school(
                    i,
                    self.vehicles[i].max_hp * 0.10 * dt,
                    true,
                    School::Fire,
                    false,
                );
                self.vehicles[i].frozen = 0.;
            }
            let hits: Vec<_> = self
                .hazards
                .iter()
                .filter(|h| {
                    (h.kind == HazardKind::Storm || distance(h.position, p) <= h.radius)
                        && h.kind != HazardKind::Tar
                        && h.age >= h.strike_at()
                        && h.age - dt < h.strike_at()
                })
                .map(|h| (h.kind, distance(h.position, p)))
                .collect();
            for (kind, d) in hits {
                match kind {
                    HazardKind::Storm => self.harm_school(
                        i,
                        if d <= 7. {
                            300_000.
                        } else {
                            300_000. / (d - 6.).max(1.)
                        },
                        true,
                        School::Nature,
                        false,
                    ),
                    HazardKind::Frost => {
                        self.harm_school(i, 150_000., true, School::Frost, false);
                        self.vehicles[i].frozen = 60.;
                    }
                    HazardKind::Fire => self.harm_school(i, 150_000., true, School::Fire, false),
                    _ => {}
                }
            }
        }
        for h in &mut self.hazards {
            if h.kind == HazardKind::Frost && h.age >= 10. {
                h.age = -1.;
                h.target = (h.target + 1) % 6;
            }
        }
        self.harm_exposed_riders(dt, vents_pulse);
        for h in &self.hazards {
            if h.kind == HazardKind::Tar && h.burning {
                for a in &mut self.adds {
                    if distance(a.position, h.position) < h.radius {
                        a.hp -= 6_500. * data::GEAR_SCALE * dt;
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
        let mut rider_damage = vec![0.; self.riders.len()];
        for a in self.adds.iter_mut().filter(|a| a.hp > 0.) {
            if let Some((rider, i, position)) = self
                .vehicles
                .iter()
                .enumerate()
                .filter(|(_, v)| v.hp > 0.)
                .map(|(i, v)| (false, i, v.body.position))
                .chain(
                    self.riders
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| r.state == RiderState::Ground && r.hp > 0.)
                        .map(|(i, r)| (true, i, r.position)),
                )
                .min_by(|(_, _, a1), (_, _, a2)| {
                    distance(*a1, a.position).total_cmp(&distance(*a2, a.position))
                })
            {
                let slow = self.hazards.iter().any(|h| {
                    h.kind == HazardKind::Tar && distance(h.position, a.position) < h.radius
                });
                a.position = toward(a.position, position, dt * if slow { 1.5 } else { 4. });
                a.attack = (a.attack - dt).max(0.);
                if distance(a.position, position) < 5. && a.attack == 0. {
                    if rider {
                        rider_damage[i] += 3_845.;
                    } else {
                        damage[i] += 3_845.;
                    }
                    a.attack = 2.;
                }
            }
        }
        for (i, d) in damage.into_iter().enumerate() {
            self.harm_school(i, d, true, School::Nature, false);
        }
        for (r, d) in self.riders.iter_mut().zip(rider_damage) {
            r.hp = (r.hp - d).max(0.);
            if r.hp == 0. {
                r.state = RiderState::Lost;
            }
        }
        self.release_destroyed_carriers();
        self.adds.retain(|a| a.hp > 0.);
        if self.boss.hp <= 0. {
            self.outcome = Outcome::Victory;
            self.say("Victoire ! F5 pour recommencer.");
        } else if self.health_fraction() <= 0. {
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
            self.storm_timer = 90.;
            for i in 0..8 {
                let id = self.id();
                let angle = (id as f32 * 2.39996) % TAU;
                self.hazards.push(Hazard {
                    id,
                    kind: HazardKind::Storm,
                    position: Point {
                        x: angle.cos() * 90.,
                        z: (angle * 1.7).sin() * 90.,
                    },
                    radius: 7.,
                    age: 0.,
                    duration: 10. + (id % 15) as f32,
                    burning: false,
                    target: i,
                });
            }
        }
        if self.fire_timer <= 0. {
            self.fire_timer += 2.;
            let id = self.id();
            self.hazards.push(Hazard {
                id,
                kind: HazardKind::Fire,
                position: self.fire_position,
                radius: 9.,
                age: 0.,
                duration: 30.8,
                burning: false,
                target: 0,
            });
        }
        if self.time >= 42. {
            let (x, z) = data::FIRE_PATH[self.fire_phase as usize];
            let goal = Point { x, z };
            self.fire_position = toward(self.fire_position, goal, 2.5 * dt);
            if distance(self.fire_position, goal) < 0.1 {
                self.fire_phase = (self.fire_phase + 1) % data::FIRE_PATH.len() as u32;
            }
        }
        if self.frost_timer <= 0. {
            self.frost_timer = f32::MAX;
            for (target, (x, z)) in [(73.4, -43.5), (-48., 75.)].into_iter().enumerate() {
                let id = self.id();
                self.hazards.push(Hazard {
                    id,
                    kind: HazardKind::Frost,
                    position: Point { x, z },
                    radius: 10.,
                    age: -1.,
                    duration: f32::MAX,
                    burning: false,
                    target,
                });
            }
        }
        if self.life_timer <= 0. {
            self.life_timer = 29.;
            for (x, z) in data::LIFE_CORNERS {
                for ward in [true, false] {
                    let id = self.id();
                    self.adds.push(Add {
                        id,
                        position: bounded(Point { x, z }),
                        hp: if ward { 302_400. } else { 64_260. },
                        ward,
                        attack: 2.,
                    });
                }
            }
            self.say("Freya : gardiens et flagellants dans les quatre angles.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn turrets_do_not_bank_attacks_before_a_late_boarding() {
        let mut f = Fight::new(Role::DemoPassenger, true);
        f.start();
        advance(&mut f, 45.);
        f.riders.push(Rider {
            id: 901,
            home: 0,
            position: f.boss.position,
            state: RiderState::Turret,
            hp: data::RIDER_HP,
        });
        f.vehicles[0].passenger = false;
        advance(&mut f, 0.5);
        assert_eq!(f.player_rider().unwrap().hp, data::RIDER_HP);
    }
    #[test]
    fn off_gcd_mortar_ignores_but_preserves_the_active_passenger_gcd() {
        let mut f = playing(Role::DemoPassenger);
        f.boss.position = Point { x: -18., z: 0. };
        f.cast(3, None).unwrap();
        let gcd = f.player().gcd[1];
        assert!(f.cast(0, Some(f.boss.position)).is_ok());
        assert_eq!(f.player().gcd[1], gcd);
    }
    #[test]
    fn passenger_survives_original_vehicle_death_during_flight() {
        let mut f = playing(Role::DemoPassenger);
        f.vehicles[0].passenger = false;
        f.fire(0, f.boss.position, ShotKind::Passenger, false);
        f.vehicles[0].hp = 0.;
        f.tick(0.02);
        assert_eq!(f.outcome, Outcome::Fighting);
    }
    #[test]
    fn ground_and_carried_passengers_are_not_immune_to_fire() {
        for state in [RiderState::Ground, RiderState::Carried] {
            let mut f = playing(Role::DemoPassenger);
            let p = Point { x: 100., z: 100. };
            f.vehicles[0].passenger = false;
            f.riders.push(Rider {
                id: 901,
                home: 0,
                position: p,
                state,
                hp: data::RIDER_HP,
            });
            if state == RiderState::Carried {
                f.vehicles[4].cargo = Some(901);
                f.vehicles[4].body.position = p;
                f.vehicles[4].frozen = 10.;
            }
            f.hazards.push(Hazard {
                id: 902,
                kind: HazardKind::Fire,
                position: p,
                radius: 9.,
                age: 2.,
                duration: 30.,
                burning: false,
                target: 0,
            });
            f.tick(0.05);
            assert!(f.player_rider().unwrap().hp < data::RIDER_HP, "{state:?}");
        }
    }
    #[test]
    fn freyas_plants_attack_a_nearer_player_on_the_ground() {
        let mut f = playing(Role::DemoPassenger);
        for v in &mut f.vehicles {
            v.body.position = Point { x: 80., z: 80. };
        }
        f.vehicles[0].passenger = false;
        let p = Point { x: 0., z: 0. };
        f.riders.push(Rider {
            id: 901,
            home: 0,
            position: p,
            state: RiderState::Ground,
            hp: data::RIDER_HP,
        });
        f.adds.push(Add {
            id: 902,
            position: p,
            hp: 64_260.,
            ward: false,
            attack: 0.,
        });
        f.tick(0.02);
        assert!(f.player_rider().unwrap().hp < data::RIDER_HP);
    }
    #[test]
    fn carrier_death_releases_the_passenger_to_the_ground() {
        let mut f = playing(Role::DemoPassenger);
        f.vehicles[0].passenger = false;
        f.vehicles[4].cargo = Some(901);
        f.vehicles[4].hp = 0.;
        f.riders.push(Rider {
            id: 901,
            home: 0,
            position: f.vehicles[4].body.position,
            state: RiderState::Carried,
            hp: data::RIDER_HP,
        });
        f.tick(0.02);
        assert_eq!(f.player_rider().unwrap().state, RiderState::Ground);
        assert_eq!(f.vehicles[4].cargo, None);
    }
    #[test]
    fn motorcycle_can_drop_a_passenger_whose_home_was_destroyed() {
        let mut f = playing(Role::Moto);
        f.vehicles[0].passenger = false;
        f.vehicles[0].hp = 0.;
        f.vehicles[4].cargo = Some(901);
        f.riders.push(Rider {
            id: 901,
            home: 0,
            position: f.player().body.position,
            state: RiderState::Carried,
            hp: data::RIDER_HP,
        });
        assert!(f.cast(3, None).is_ok());
        assert_eq!(f.vehicles[4].cargo, None);
        assert_eq!(f.riders[0].state, RiderState::Ground);
    }
    #[test]
    fn offensive_ai_can_position_in_the_expanded_half_of_the_arena() {
        let mut f = playing(Role::Moto);
        f.boss.position = Point { x: 90., z: 0. };
        f.boss.target = 2;
        f.vehicles[0].body.position = Point { x: 58., z: 0. };
        let old = f.vehicles[0].body.position;
        f.ai(0, 0.02);
        assert_eq!(f.vehicles[0].body.position, old);
    }
    #[test]
    fn ground_passenger_can_move_but_boarded_and_carried_passengers_cannot() {
        let mut f = playing(Role::DemoPassenger);
        assert!(!f.player_can_move());
        f.vehicles[0].passenger = false;
        f.riders.push(Rider {
            id: 901,
            home: 0,
            position: f.boss.position,
            state: RiderState::Ground,
            hp: data::RIDER_HP,
        });
        assert!(f.player_can_move());
        f.riders[0].state = RiderState::Carried;
        assert!(!f.player_can_move());
        f.riders[0].state = RiderState::Turret;
        assert!(!f.player_can_move());
    }
    #[test]
    fn hm_and_equipment_use_absolute_health() {
        let normal = Fight::new(Role::DemoDriver, false);
        let hard = Fight::new(Role::DemoDriver, true);
        assert_eq!(normal.boss.hp, 23_009_250.);
        assert!((hard.boss.hp - 88_392_336.).abs() < 16.);
        assert_eq!(normal.vehicles[0].hp, 913_500.);
        assert_eq!(normal.vehicles[2].hp, 1_644_300.);
        assert_eq!(normal.vehicles[4].hp, 730_800.);
    }
    #[test]
    fn ai_passenger_boosts_a_human_driver_and_pays_pyrite() {
        let mut f = playing(Role::DemoDriver);
        f.boss.target = 0;
        f.boss.pursuit = 25.;
        f.boss.position = f.player().body.position;
        f.ai(0, 0.02);
        assert_eq!(f.player().boost, 20.);
        assert_eq!(f.player().pyrite, 75.);
        f.ai(0, 0.02);
        assert_eq!(f.player().pyrite, 75.);
    }
    #[test]
    fn absent_passenger_cannot_boost_an_ai_driver() {
        let mut f = playing(Role::SiegeDriver);
        f.vehicles[0].passenger = false;
        f.boss.target = 0;
        f.boss.pursuit = 25.;
        f.boss.position = f.vehicles[0].body.position;
        f.ai(0, 0.02);
        assert_eq!(f.vehicles[0].boost, 0.);
        assert_eq!(f.vehicles[0].pyrite, 100.);
    }
    #[test]
    fn boost_uses_twenty_seconds_and_twenty_five_pyrite() {
        let mut f = playing(Role::DemoPassenger);
        f.cast(3, None).unwrap();
        assert_eq!(f.player().boost, 20.);
        assert_eq!(f.player().pyrite, 75.);
    }
    #[test]
    fn vehicle_global_cooldown_blocks_different_driver_spells() {
        let mut f = playing(Role::DemoDriver);
        f.cast(0, Some(f.boss.position)).unwrap();
        assert!(f.cast(1, Some(f.boss.position)).is_err());
        assert_eq!(f.player().pyrite, 100.);
    }
    #[test]
    fn displayed_cooldowns_include_the_seat_global_cooldown() {
        let mut f = playing(Role::DemoDriver);
        f.cast(0, Some(f.boss.position)).unwrap();
        assert_eq!(f.cooldowns()[0], 1.);
        assert_eq!(f.cooldowns()[1], 1.);
        assert!(f.cast(0, Some(f.boss.position)).is_err());
        advance(&mut f, 1.1);
        assert_eq!(f.cooldowns()[0], 0.);
        assert!(f.cast(0, Some(f.boss.position)).is_ok());
    }
    #[test]
    fn displayed_cooldowns_preserve_off_gcd_spells_and_seat_independence() {
        let mut f = playing(Role::SiegeGunner);
        f.cast(1, Some(f.boss.position)).unwrap();
        assert_eq!(f.cooldowns()[0], 0.);
        assert_eq!(f.cooldowns()[1], 1.);
        assert_eq!(f.cooldowns()[2], 1.);
        f.role = Role::SiegeDriver;
        assert_eq!(f.cooldowns(), [0.; 5]);
    }
    #[test]
    fn loading_the_catapult_disables_the_passenger_boost() {
        let mut f = playing(Role::DemoDriver);
        f.vehicles[0].loaded = true;
        f.boss.target = 0;
        f.ai(0, 0.02);
        assert_eq!(f.player().boost, 0.);
        assert_eq!(f.player().pyrite, 100.);
    }
    #[test]
    fn speed_stacks_survive_a_pursuit_change_but_not_shutdown() {
        let mut f = playing(Role::DemoDriver);
        f.boss.speed_stacks = 7;
        f.boss.pursuit = 0.;
        f.tick(0.02);
        assert_eq!(f.boss.speed_stacks, 7);
        f.boss.turrets = [0.; 2];
        f.tick(0.02);
        assert_eq!(f.boss.speed_stacks, 0);
        assert_eq!(f.boss.shutdown, 20.);
    }
    #[test]
    fn shield_absorbs_fifteen_percent_but_not_nature_damage() {
        let mut f = playing(Role::SiegeGunner);
        f.player_mut().shield = 5.;
        let hp = f.player().hp;
        f.harm_school(2, 1_000., false, School::Fire, false);
        assert_eq!(f.player().hp, hp - 850.);
        f.harm_school(2, 1_000., false, School::Nature, false);
        assert_eq!(f.player().hp, hp - 1_850.);
    }
    #[test]
    fn manual_turret_attack_rejects_a_stale_vehicle_aim() {
        let mut f = playing(Role::DemoPassenger);
        f.vehicles[0].passenger = false;
        f.riders.push(Rider {
            id: 901,
            home: 0,
            position: f.boss.position,
            state: RiderState::Turret,
            hp: data::RIDER_HP,
        });
        assert!(f.cast(0, Some(f.boss.position)).is_err());
        assert!(!f.attacking_turrets());
        let turrets = f.boss.turrets;
        advance(&mut f, 1.);
        assert_eq!(f.boss.turrets, turrets);
    }
    #[test]
    fn two_seats_do_not_share_global_cooldowns() {
        let mut f = playing(Role::DemoDriver);
        f.cast(0, Some(f.boss.position)).unwrap();
        assert!(f.cast_for(0, Role::DemoPassenger, 3, None, false).is_ok());
        assert_eq!(f.player().boost, 20.);
    }
    #[test]
    fn empty_pyrite_cannot_fund_an_ai_boost() {
        let mut f = playing(Role::DemoDriver);
        f.player_mut().pyrite = 24.;
        f.boss.target = 0;
        f.ai(0, 0.02);
        assert_eq!(f.player().boost, 0.);
        assert_eq!(f.player().pyrite, 24.);
    }
    #[test]
    fn actual_hm_attempt_ends_without_smoke_test_health_or_ammo() {
        let mut f = Fight::new(Role::DemoPassenger, true);
        f.start();
        for _ in 0..(60 * 600) {
            f.tick(1. / 60.);
            if f.outcome != Outcome::Fighting {
                break;
            }
        }
        assert!(matches!(f.outcome, Outcome::Victory | Outcome::Defeat));
        assert!(f.time > 30. && f.time < 600.);
        assert!(
            f.vehicles
                .iter()
                .all(|v| v.hp >= 0. && v.hp <= v.max_hp && v.pyrite >= 0. && v.pyrite <= 100.)
        );
    }
    #[test]
    fn a_single_human_passenger_can_attack_both_turrets() {
        let mut f = Fight::new(Role::DemoPassenger, true);
        f.start();
        f.vehicles[0].passenger = false;
        f.riders.push(Rider {
            id: 900,
            home: 0,
            position: f.boss.position,
            state: RiderState::Turret,
            hp: data::RIDER_HP,
        });
        advance(&mut f, 2.);
        assert_eq!(f.stats.shutdowns, 0);
        assert!(f.cast(0, None).is_ok());
        advance(&mut f, 39.);
        assert_eq!(
            f.stats.shutdowns, 1,
            "t={} outcome={:?} turrets={:?} riders={:?}",
            f.time, f.outcome, f.boss.turrets, f.riders
        );
    }
    #[test]
    fn original_vehicle_death_does_not_kill_a_passenger_on_the_boss() {
        let mut f = playing(Role::DemoPassenger);
        f.vehicles[0].passenger = false;
        f.vehicles[0].hp = 0.;
        f.riders.push(Rider {
            id: 900,
            home: 0,
            position: f.boss.position,
            state: RiderState::Turret,
            hp: data::RIDER_HP,
        });
        f.tick(0.02);
        assert_eq!(f.outcome, Outcome::Fighting);
    }
    #[test]
    fn nonfire_projectiles_do_not_thaw_hodirs_prison() {
        let mut f = playing(Role::DemoDriver);
        f.vehicles[0].frozen = 60.;
        let p = f.vehicles[0].body.position;
        f.fire(2, p, ShotKind::Rocket, false);
        let shot = f.shots.pop().unwrap();
        f.impact(shot);
        assert_eq!(f.vehicles[0].frozen, 60.);
        f.fire(2, p, ShotKind::Mortar, false);
        let shot = f.shots.pop().unwrap();
        f.impact(shot);
        assert_eq!(f.vehicles[0].frozen, 0.);
    }
    #[test]
    fn flame_vents_deal_damage_during_the_channel() {
        let mut f = playing(Role::DemoDriver);
        for v in &mut f.vehicles[1..] {
            v.hp = 0.;
        }
        f.boss.position = f.player().body.position;
        f.boss.position.z -= 35.;
        f.boss.vents = 0.;
        let hp = f.player().hp;
        advance(&mut f, 1.2);
        assert!(f.boss.cast > 8.);
        assert!(f.player().hp < hp - 3000.);
    }
    #[test]
    fn difficulty_switch_rebuilds_health_only_before_pull() {
        let mut f = Fight::new(Role::DemoDriver, true);
        f.set_towers(false);
        assert_eq!(f.boss.hp, 23_009_250.);
        f.start();
        f.set_towers(true);
        assert!(!f.towers);
        assert_eq!(f.boss.hp, f.boss.max_hp);
    }
    #[test]
    fn frost_ray_can_reach_a_stationary_target() {
        let mut f = playing(Role::DemoDriver);
        f.vehicles[0].hp = 913_500.;
        f.towers = true;
        f.frost_timer = 100.;
        f.storm_timer = 100.;
        f.fire_timer = 100.;
        f.life_timer = 100.;
        let p = f.player().body.position;
        f.hazards.push(Hazard {
            id: 900,
            kind: HazardKind::Frost,
            position: Point {
                x: p.x + 1.,
                z: p.z,
            },
            radius: 10.,
            age: -1.,
            duration: f32::MAX,
            burning: false,
            target: 0,
        });
        for v in &mut f.vehicles[1..] {
            v.hp = 0.;
        }
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
        let hp = f.player().hp;
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
        assert!((f.player().hp - (hp - hp * 0.10 * 0.05)).abs() < 0.1);
    }
    #[test]
    fn every_role_spell_can_be_cast_with_its_preconditions() {
        for role in Role::ALL {
            for (slot, spec) in spells(role).iter().enumerate() {
                let mut f = playing(role);
                let owner = role.index();
                f.vehicles[owner].body.position = Point { x: 0., z: 0. };
                f.boss.position = Point { x: 0., z: -12. };
                f.vehicles[owner].loaded = role == Role::DemoDriver;
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
                assert_eq!(f.cooldowns()[slot], spec.cooldown.max(spec.gcd));
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
        for home in [0, 1] {
            f.vehicles[home].passenger = false;
            f.riders.push(Rider {
                id: 900 + home as u64,
                home,
                position: f.boss.position,
                state: RiderState::Turret,
                hp: data::RIDER_HP,
            });
        }
        advance(&mut f, 23.);
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
        assert_eq!(f.boss.hp, 23_009_250.);
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
            for x in [-103_f32, 103.] {
                for z in [-103_f32, 103.] {
                    let mut f = playing(Role::DemoDriver);
                    f.vehicles[index].body.position = Point { x, z };
                    f.boss.position = Point {
                        x: x.signum() * 100.,
                        z: z.signum() * 100.,
                    };
                    for _ in 0..200 {
                        f.ai(index, 0.02);
                    }
                    let p = f.vehicles[index].body.position;
                    assert!(distance(p, Point { x, z }) > 8., "{index} stuck at {x}/{z}");
                    assert!(p.x.abs() <= 103. && p.z.abs() <= 103.);
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
        assert_eq!(f.boss.hp, 23_009_250.);
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
        f.boss.position.x += 30.;
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
        assert_eq!(f.player().energy, 62.);
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
        assert_eq!(f.boss.hp, 23_009_250.);
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
        advance(&mut f, 4.);
        assert_eq!(f.riders[0].hp, data::RIDER_HP);
        f.vehicles[4].body.position = f.vehicles[0].body.position;
        f.vehicles[4].cooldowns[3] = 0.;
        f.vehicles[4].gcd[0] = 0.;
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
            && v.body.position.x.abs() <= 105.5
            && v.body.position.z.abs() <= 105.5));
        assert!(f.boss.hp < 23_009_250.);
    }
}
