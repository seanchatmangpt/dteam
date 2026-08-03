# 19 — DTEAM Globe Math (unios-only)

## Principle

DTEAM sees the world as:

```
canonical facts → process challenge → verified result + receipt
```

DTEAM never imports unibit. DTEAM never imports unios. DTEAM depends only on
a narrow trait — `DTeamUnios` — implemented privately by `dteam-unios`.

## Crate layout

```
crates/
├── dteam-globe/            (globe math: coords, trajectories, planes, motion, projection)
├── dteam-supply-chain/     (scenarios, planner using globe semantics)
├── dteam-process/          (process intent)
├── dteam-receipt/          (proof surface)
└── dteam-unios/            (DTEAM-facing unios adapter trait)
```

Only `dteam-unios` imports unios. Everything else imports DTEAM types.

## The adapter trait

```rust
pub trait DTeamUnios {
    fn admit_globe_motion(&mut self, motion: GlobeMotion) -> GlobeAdmission;
    fn project_globe(&self, request: GlobeProjectionRequest) -> GlobeProjection;
    fn verify_receipt(&self, receipt: &DTeamReceipt) -> VerificationStatus;
}

pub struct GlobeAdmission {
    pub admitted: bool,
    pub receipt: DTeamReceipt,
    pub projection: Option<GlobeProjection>,
}
```

## GlobeCell

Semantic coordinate (not substrate address):

```rust
pub struct GlobeCell {
    pub domain: DomainId,        // u8
    pub cell: AttentionCell,     // u16 — 64² attention cell
    pub place: LocalPlace,       // u8  — local process place in 64³
}

impl GlobeCell {
    pub const fn new(domain: u8, cell: u16, place: u8) -> Self { ... }
    pub const fn index_64_3(self) -> u32 {
        ((self.domain.0 as u32) << 18)
          | ((self.cell.0 as u32) << 6)
          | (self.place.0 as u32)
    }
}
```

Geographic → globe cell:

```rust
pub fn geo_to_globe_cell(coord: GeoCoord, domain: u8, place: u8) -> GlobeCell {
    let lat_norm = ((coord.lat + 90.0) / 180.0).clamp(0.0, 0.999999);
    let lon_norm = ((coord.lon + 180.0) / 360.0).clamp(0.0, 0.999999);
    let cell = (lat_norm * 8.0) as u8 * 8 + (lon_norm * 8.0) as u8;
    GlobeCell::new(domain, cell as u16, place)
}
```

## Semantic planes

```rust
pub enum GlobePlane {
    RouteCurrent, RouteExpected,
    Law, Capability, Risk, Reward,
    Scenario(ScenarioKind),
    Projection(String),
    Audit,
}

pub enum ScenarioKind {
    Storm, PortStrike, SupplierFailure, Sanctions,
    FuelShock, DemandSpike, Custom(String),
}
```

## Trajectory

```rust
pub struct ShipmentTrajectory {
    pub shipment: ShipmentId,
    pub segments: Vec<RouteSegment>,
}

pub struct RouteSegment {
    pub from: GlobeCell,
    pub to: GlobeCell,
    pub plane: GlobePlane,
}
```

A shipment is not a row. A shipment is a **trajectory through lawful globe coordinates**.

## GlobeMotion

```rust
pub struct GlobeMotion {
    pub id: GlobeMotionId,
    pub kind: GlobeMotionKind,
    pub requires: Vec<GlobePredicate>,
    pub produces: Vec<GlobePredicate>,
    pub consumes: Vec<GlobePredicate>,
    pub law: Vec<GlobePredicate>,
    pub capability: Vec<GlobePredicate>,
    pub scenario: Vec<GlobePredicate>,
}

pub enum GlobeMotionKind {
    ScenarioUpdate { scenario: ScenarioKind, affected: Vec<GlobeCell> },
    Reroute { shipment: ShipmentId, candidate: Vec<RouteSegment> },
    AdmitRouteSegment { segment: RouteSegment },
    BlockCell { cell: GlobeCell, reason: String },
    ClearCell { cell: GlobeCell },
}
```

## Example: storm reroute flow

```rust
pub fn storm_reroute_flow<U: DTeamUnios>(unios: &mut U) {
    let port_a = geo_to_globe_cell(GeoCoord { lat: 34.05, lon: -118.24 }, 1, 7);
    let port_b = geo_to_globe_cell(GeoCoord { lat: 35.67, lon: 139.65 }, 1, 9);
    let storm = storm_update_motion(GlobeMotionId(100), vec![port_a]);
    let storm_result = unios.admit_globe_motion(storm);
    assert!(storm_result.admitted);
    let reroute = SupplyChainPlanner::reroute_motion(
        GlobeMotionId(101),
        ShipmentId("shipment-42".to_string()),
        vec![RouteSegment { from: port_a, to: port_b, plane: GlobePlane::RouteCurrent }],
    );
    let reroute_result = unios.admit_globe_motion(reroute);
    let projection = unios.project_globe(GlobeProjectionRequest {
        projection: "executive_supply_chain".to_string(),
        scenario: Some("storm".to_string()),
    });
    assert_eq!(unios.verify_receipt(&reroute_result.receipt), VerificationStatus::Verified);
}
```

## Boundary enforcement

**Forbidden in public DTEAM code:**
```rust
use unios::Something;       // NO
use unibit::Something;      // NO
```

**Required:**
```rust
use dteam_unios::DTeamUnios;
```

Public README says: "DTEAM is a black-box deterministic process intelligence
engine. It returns verified process results with receipts."

Internal engineering says: "DTEAM compiles semantic process motion. Private
adapter lowers to substrate handles."
