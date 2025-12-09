use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Top-level structure of a LEF library.
#[derive(Default, Clone, Debug)]
pub struct LefTechnology {
    /// LEF version.
    pub version: Option<f64>,
    /// Characters used to indicate bus bits. Default is `[` and `]`.
    pub busbitchars: (char, char),
    /// Character used as path separator. Default is `/`.
    pub dividerchar: char,
    /// Units used in this library.
    pub units: LefUnits,
    /// Grid for geometrical alignment. Cells and shapes snap to locations on this grid.
    pub manufacturing_grid: Option<f64>,
    /// Type of distance measure (Euclidean: `dx^2 + dy^2`, MaxXY: `max(dx, dy)`)
    pub clearance_measure: LefClearanceMeasure,

    /// Definitions of custom properties.
    pub property_definitions: HashMap<String, ()>,

    /// Layer definitions (masterslice, cut, routing, ...).
    /// Layers are defined in their process order from bottom to top.
    pub layers: Vec<LefLayer>,

    /// Maximum number of single-cut vias stacked on top of each other.
    /// Optionally defines a range of (bottom layer, top layer) where the rule applies. Otherwise
    /// the rule applies to all layers.
    pub max_via_stack: Option<(u64, Option<(String, String)>)>,

    /// Definitions of fixed VIAs by name.
    pub vias: HashMap<String, LefVia>,
    pub via_rules: HashMap<String, LefViaRule>,

    /// NONDEFAULTRULEs by name.
    pub non_default_rule: (),

    /// All SITE definitions by name.
    pub sites: HashMap<String, LefSiteDefinition>,
}

impl LefTechnology {
    pub fn new() -> Self {
        LefTechnology {
            version: None,
            busbitchars: ('[', ']'),
            dividerchar: '/',
            ..Default::default()
        }
    }
}

/// Units used in the library.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct LefUnits {
    /// Time in nano seconds.
    pub time_ns: u64,
    /// Capacitance in pico farads.
    pub capacitance_pf: u64,
    /// Resistance in ohms.
    pub resistance_ohms: u64,
    /// Power in milli watts.
    pub power_mw: u64,
    /// Current in milli amperes.
    pub current_ma: u64,
    /// Voltage in volts.
    pub voltage_v: u64,
    /// Length in micro meters.
    pub database_microns: u64,
    /// Frequency in mega hertz.
    pub frequency_mega_hz: u64,
}

/// Macro SITE declaration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LefSite {
    /// Name of the site.
    pub name: String,
    /// Origin of the site within the macro. Unit is microns.
    pub origin: (f64, f64),
    /// Orientation of the site.
    pub site_orient: LefOrient,
    /// Optional repetition pattern.
    pub step_pattern: Option<LefStepPattern>,
}

/// SITE definition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LefSiteDefinition {
    /// Name of the site.
    pub name: String,
    /// Dimensions of the site.
    pub size: (f64, f64),
    /// Define orientations of the site that are considered equivalent.
    /// This is used for example to specify whether cell flipping inside
    /// a row is allowed.
    pub symmetry: LefSymmetry,
    /// Specify site type (IO or CORE).
    pub class: LefSiteClass,
    /// Construct this site as a composition of previously defined sites.
    /// List of tuples: (previous site, orientation)
    pub row_pattern: Vec<(String, LefOrient)>,
}

/// Array-like repetition of an element.
///
/// Use `each_offset()` to iterate through all offsets described by this pattern.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LefStepPattern {
    /// Number of repetitions in x-direction.
    pub num_x: u64,
    /// Number of repetitions in y-direction.
    pub num_y: u64,
    /// Spacing in x-direction.
    pub space_x: f64,
    /// Spacing in y-direction.
    pub space_y: f64,
}

impl LefStepPattern {
    /// Return an iterator over each offset of the step pattern.
    /// The origin is at (0.0, 0.0).
    pub fn each_offset(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        (0..self.num_x)
            .flat_map(move |x| (0..self.num_y).map(move |y| (x, y)))
            .map(move |(x, y)| (x as f64 * self.space_x, y as f64 * self.space_y))
    }
}

impl Default for LefStepPattern {
    fn default() -> Self {
        LefStepPattern {
            num_x: 1,
            num_y: 1,
            space_x: 0.0,
            space_y: 0.0,
        }
    }
}

/// Holds either the value of the SPACING argument or DESIGNRULEWIDTH argument of a geometrical
/// layer as used in the LAYER definition in PIN or OBS.
#[derive(Clone, Debug)]
pub enum LefSpacingOrDesignRuleWidth {
    /// Minimal allowed spacing between this shape and other shapes.
    MinSpacing(f64),
    /// Effective design rule width.
    DesignRuleWidth(f64),
}

/// Either a path, rectangle or polygon.
#[derive(Clone, Debug)]
pub enum LefShape {
    /// Width and path.
    Path(f64, Vec<(f64, f64)>),
    /// Corner points of a rectangle.
    Rect((f64, f64), (f64, f64)),
    /// Vertices of a polygon.
    Polygon(Vec<(f64, f64)>),
}

/// Shape with an optional array step pattern.
#[derive(Clone, Debug)]
pub struct LefGeometry {
    /// Array-like repetition of the shape.
    pub step_pattern: Option<LefStepPattern>,
    /// Geometric primitive.
    pub shape: LefShape,
}

#[derive(Clone, Debug)]
pub enum LefViaRule {
    Generate(LefViaGenerateRule),
    // TODO: Fixed
}

/// A generated via.
#[derive(Clone, Debug, Default)]
pub struct LefViaGenerateRule {
    /// Default via to be used for routing between the adjacent layers.
    pub is_default: bool,
    /// Via generate rule which was used to generate this via.
    pub rule_name: String,
    /// Bottom, cut and top layer.
    pub layers: (String, String, String),
    /// (bottom-x, bottom-y), (top-x, top-y) enclosure
    pub enclosure: ((f64, f64), (f64, f64)),
    /// (bottom-minwidth, bottom-maxwidth) (top-minwidth, rop-maxwidth)
    pub width: ((f64, f64), (f64, f64)),
    /// cut rectangle
    pub rect: ((f64, f64), (f64, f64)),
    /// center-to-center spacing in the x and y dimensions to create an array of contact cuts.
    pub spacing: (f64, f64),
}

/// Either a rectangle or a polygon.
#[derive(Clone, Debug)]
pub enum LefViaShape {
    /// Axis-aligned rectangle.
    Rect((f64, f64), (f64, f64)),
    /// Polygon.
    Polygon(Vec<(f64, f64)>),
}

/// An explicitly defined via.
#[derive(Clone, Debug, Default)]
pub struct LefVia {
    /// Default via to be used for routing between the adjacent layers.
    pub is_default: bool,
    /// Electrical resistance of the via.
    pub resistance: Option<f64>,
    /// Layers and shapes of the via geometry.
    pub geometry: HashMap<String, Vec<LefViaShape>>,
}

/// MACRO definition.
#[derive(Clone, Debug, Default)]
pub struct LefMacro {
    /// Name of the macro.
    pub name: String,
    /// Class of the macro.
    pub class: Option<LefMacroClass>,
    /// Disable shifting of masks.
    /// When set, shifting of macro pin mask assignments to other masks is not allowed.
    /// Used for technologies that use multi-mask patterning.
    pub fixed_mask: bool,
    /// Name of the corresponding cell layout in the GDS/OASIS file.
    /// Associated with an offset and orientation.
    pub foreign: Vec<(String, (f64, f64), LefOrient)>,
    /// Coordinate of the origin of the macro. Default is (0, 0).
    /// A placement of a cell in DEF is given by the location of the origin.
    pub origin: (f64, f64),
    /// Name of electrically equivalent macro.
    pub eeq: Option<String>,
    /// Width and height of the macro.
    pub size: Option<(f64, f64)>,
    /// Symmetry of the macro. Tells how the macro can be mirrored and rotated.
    pub symmetry: LefSymmetry,

    /// SITES associated with the macro. Normal macros have only one associated site.
    pub sites: Vec<LefSite>,

    /// Definitions of the electrical pins of the macro.
    pub pins: Vec<LefMacroPin>,
    /// Obstructions (blockages).
    pub obs: Vec<LefLayerGeometries>,
    /// Density specifications.
    pub density: Vec<()>,

    /// Additional properties of the macro.
    pub properties: HashMap<String, ()>,
}

/// PIN definition of a MACRO.
#[derive(Clone, Debug, Default)]
pub struct LefMacroPin {
    /// Name of the pin.
    pub name: String,
    /// Name of the NONDEFAULTRULE to be used when tapering wires to this pin.
    pub taper_rule: Option<String>,
    /// Signal direction.
    pub direction: Option<LefPinDirection>,
    /// Type of the signal for this pin.
    pub signal_use: Option<LefSignalUse>,

    /// Net name where this pin should be connected if it is tied HIGH (constant logical 1).
    pub supply_sensitivity: Option<String>,
    /// Net name where this pin should be connected if it is tied LOW (constant logical 0).
    pub ground_sensitivity: Option<String>,

    /// Specify special connection requirements of the pin.
    pub shape_type: Option<LefPinShape>,

    /// Name of another pin that must be connected to this pin.
    pub must_join: Option<String>,
    ///
    pub ports: Vec<LefMacroPinPort>,
}

/// PORT definition of a MACRO PIN.
/// A port describes where a pin is geometrically located.
/// A pin can have multiple ports. They are electrically equivalent.
#[derive(Clone, Debug, Default)]
pub struct LefMacroPinPort {
    /// Type of the port.
    pub class: Option<LefPortClass>,
    /// Geometrical shapes and vias that make this port.
    pub geometries: Vec<LefLayerGeometries>,
}

/// Geometrical shapes on a named layer as used in MACRO PIN and OBS definitions.
#[derive(Clone, Debug, Default)]
pub struct LefLayerGeometries {
    /// Name of the layer.
    pub layer_name: String,
    /// Obstruction blocks signal routing but not power or ground routing.
    pub except_pg_net: bool,
    /// Either minimal allowed spacing or an effective width.
    pub spacing_or_designrule_width: Option<LefSpacingOrDesignRuleWidth>,
    /// Specify the width to be used for PATH. If not specified the default with for this layer is used.
    pub width: Option<f64>,
    /// Geometrical shapes (PATH, RECT, POLYGON). Together with a repetition pattern.
    pub geometries: Vec<LefGeometry>,
    /// Specify vias to be placed with their locations.
    pub vias: Vec<()>,
}

/// Type of distance measurement
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LefClearanceMeasure {
    /// Take maximum of x or y distance.
    Maxxy,
    /// `sqrt(x^2 + y^2)`
    Euclidean,
}

impl Default for LefClearanceMeasure {
    fn default() -> Self {
        Self::Euclidean
    }
}

impl FromStr for LefClearanceMeasure {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "MAXXY" => Ok(Self::Maxxy),
            "EUCLIDEAN" => Ok(Self::Euclidean),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefClearanceMeasure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Maxxy => f.write_str("MAXXY"),
            Self::Euclidean => f.write_str("EUCLIDEAN"),
        }
    }
}

/// Preferred routing direction on a routing layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LefRoutingDirection {
    /// Vertical routing direction.
    Vertical,
    /// Horizontal routing direction.
    Horizontal,
    /// 45 degree routing direction.
    Diag45,
    /// 135 degree routing direction.
    Diag135,
}

impl FromStr for LefRoutingDirection {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "VERTICAL" => Ok(Self::Vertical),
            "HORIZONTAL" => Ok(Self::Horizontal),
            "DIAG45" => Ok(Self::Diag45),
            "DIAG135" => Ok(Self::Diag135),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefRoutingDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vertical => f.write_str("VERTICAL"),
            Self::Horizontal => f.write_str("HORIZONTAL"),
            Self::Diag45 => f.write_str("DIAG45"),
            Self::Diag135 => f.write_str("DIAG135"),
        }
    }
}

/// Type of the signal.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LefSignalUse {
    /// Data signal.
    Signal,
    /// Analog signal.
    Analog,
    /// Power supply.
    Power,
    /// Ground.
    Ground,
    /// Clock signal.
    Clock,
}

impl FromStr for LefSignalUse {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "SIGNAL" => Ok(Self::Signal),
            "ANALOG" => Ok(Self::Analog),
            "POWER" => Ok(Self::Power),
            "GROUND" => Ok(Self::Ground),
            "CLOCK" => Ok(Self::Clock),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefSignalUse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signal => f.write_str("SIGNAL"),
            Self::Analog => f.write_str("ANALOG"),
            Self::Power => f.write_str("POWER"),
            Self::Ground => f.write_str("GROUND"),
            Self::Clock => f.write_str("CLOCK"),
        }
    }
}

/// TODO: Document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LefPortClass {
    ///
    None,
    ///
    Core,
    ///
    Bump,
}

impl FromStr for LefPortClass {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "NONE" => Ok(Self::None),
            "CORE" => Ok(Self::Core),
            "BUMP" => Ok(Self::Bump),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefPortClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("NONE"),
            Self::Core => f.write_str("CORE"),
            Self::Bump => f.write_str("BUMP"),
        }
    }
}

/// Type of the pin shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LefPinShape {
    ///
    Abutment,
    ///
    Ring,
    ///
    Feedthru,
}

impl FromStr for LefPinShape {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "ABUTMENT" => Ok(Self::Abutment),
            "RING" => Ok(Self::Ring),
            "FEEDTHRU" => Ok(Self::Feedthru),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefPinShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Abutment => f.write_str("ABUTMENT"),
            Self::Ring => f.write_str("RING"),
            Self::Feedthru => f.write_str("FEEDTHRU"),
        }
    }
}

/// Spacing rules for a routing layer.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct LefSpacingRules {
    pub min_spacing: f64,
    pub spacing_type: Option<LefSpacingType>,
}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum LefSpacingType {
    Range {
        min_width: f64,
        max_width: f64,
        spacing_range_type: Option<LefSpacingRangeType>,
    },
    EndOfLine {
        eol_width: f64,
        eol_widthing: f64,
    },
    /// Rule applies only for two shapes of the same net.
    SameNet {
        /// PGONLY, same-net rule applies for power and ground nets only.
        power_ground_only: bool,
    },
    NotchLength {
        min_notch_length: f64,
    },
    EndOfNotchWidth {
        end_of_notch_width: f64,
        min_notch_spacing: f64,
        min_notch_length: f64,
    },
}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum LefSpacingRangeType {
    UseLengthThreshold,
    Influence { influence_length: f64 },
}

/// SPACINGTABLE, spacing rules for a routing layer.
#[derive(Clone, Debug, Default)]
pub struct LefSpacingTable {
    /// Indices of the table columns.
    pub parallel_run_lengths: Vec<f64>,
    /// Indices of the table rows.
    pub widths: Vec<f64>,
    /// Table values
    pub spacings: Vec<Vec<f64>>,
}

/// Layer definition.
/// A layer can have different types:
///
/// * MasterSlice: This is usually the first layer in the stack.
/// * Cut: Via layer that connects the previous and next layer.
/// * Routing: Metal wires.
#[derive(Clone, Debug)]
pub enum LefLayer {
    /// MASTERSLICE (poly) layer.
    MasterSlice(LefMasterSliceLayer),
    /// CUT layer.
    Cut(LefCutLayer),
    /// ROUTING layer.
    Routing(LefRoutingLayer),
}

impl LefLayer {
    /// Get the name of the layer.
    pub fn name(&self) -> &String {
        match self {
            LefLayer::MasterSlice(l) => &l.name,
            LefLayer::Cut(l) => &l.name,
            LefLayer::Routing(l) => &l.name,
        }
    }
}

/// Design rules for a MASTERSLICE or OVERLAP layer.
/// Master slice layers are usually polysilicon layers and are typically used when a MACRO has
/// pins on the poly layer.
#[derive(Clone, Debug, Default)]
pub struct LefMasterSliceLayer {
    /// Name of the masterslice layer.
    pub name: String,
    /// Number of masks used for double- or triple-patterning.
    pub mask_num: Option<u32>,

    /// Custom properties.
    pub properties: HashMap<String, LefPropertyValue>,
    // TODO: PROPERTY_LEF58_TYPE, PROPERTY_LEF58_TRIMMEDMETAL
}

/// Design rules for a CUT (via) layer.
#[derive(Clone, Debug, Default)]
pub struct LefCutLayer {
    /// Name of the cut layer.
    pub name: String,
    /// Number of masks used for double- or triple-patterning.
    pub mask_num: Option<u32>,
    /// Minimum spacing rules between cuts of same or different nets.
    pub spacing: Vec<LefCutSpacingRule>,
    /// Spacing table to be used on this cut layer.
    pub spacing_table: Option<()>,
    /// TODO
    pub array_spacing: Option<()>,
    /// Minimum width of a cut in microns.
    /// Usually this is the only allowed size of a cut.
    pub width: Option<f64>,
    /// Enclosure rules that must be met.
    pub enclosure: Vec<LefEnclosureRule>,
    /// Preferred enclosure rules that can be used to improve yield but must not necessarily
    /// be met.
    pub prefer_enclosure: Vec<LefEnclosureRule>,
    /// Resistance per cut.
    pub resistance: Option<f64>,
    /// Custom properties.
    pub properties: HashMap<String, LefPropertyValue>,
    // TODO: Antenna rule definitions.
}

/// ENCLOSURE rules for a CUT (via) layer.
#[derive(Clone, Debug)]
pub struct LefEnclosureRule {
    /// Rule applies for the routing layer above.
    pub above: bool,
    /// Rule applies for the routing layer below.
    pub below: bool,
    /// Adjacent routing layers must overhang on the two opposing sides (in x-direction (?)).
    pub overhang1: f64,
    /// Adjacent routing layers must overhang on the two opposing sides (in y-direction (?)).
    pub overhang2: f64,
    /// Rule only applies if the width of the adjacent shape of the routing layer is greater or equal
    /// than `min_width`.
    /// Default is 0.
    pub min_width: f64,
    /// TODO
    /// Don't use the WIDTH rule when another via is present to the current via within this distance.
    pub except_extracut_within: f64,
    /// Rule only applies if the total length of the longest overhangs is greater or equal
    /// to `min_length`.
    /// The overhang length is measured from the via cut center.
    /// Default is 0.
    pub min_length: f64,
}

impl Default for LefEnclosureRule {
    fn default() -> Self {
        Self {
            above: true,
            below: true,
            overhang1: 0.0,
            overhang2: 0.0,
            min_width: 0.0,
            except_extracut_within: 0.0,
            min_length: 0.0,
        }
    }
}

/// SPACING rules for a CUT (via) layer.
#[derive(Clone, Debug)]
pub struct LefCutSpacingRule {
    /// Spacing between cuts.
    pub spacing: f64,
    /// Measure the spacing from center of the cut to the center of another cut
    /// instead of from edge to edge.
    /// This is enabled by default.
    pub center_to_center: bool,
    /// Tell if this spacing rule applies for same-net cuts.
    pub same_net: bool,
}

impl Default for LefCutSpacingRule {
    fn default() -> Self {
        Self {
            spacing: 0.0,
            center_to_center: true,
            same_net: false,
        }
    }
}

/// Design rules for a routing layer.
#[derive(Clone, Debug)]
pub struct LefRoutingLayer {
    /// Name of the routing layer.
    pub name: String,
    /// Number of masks used for double- or triple-patterning.
    pub mask_num: Option<u32>,
    /// Preferred routing direction.
    pub direction: LefRoutingDirection,
    /// Routing pitch in x and y direction in microns.
    pub pitch: (f64, f64),
    /// Routing pitch for diagonal directions in microns.
    pub diag_pitch: Option<(f64, f64)>,
    /// Default wire width in microns.
    pub width: f64,
    ///
    pub offset: Option<(f64, f64)>,
    /// Default width for diagonal wires in microns.
    pub diag_width: Option<f64>,
    /// Default spacing for diagonal wires in microns.
    pub diag_spacing: Option<f64>,
    /// Minimum edge length for diagonal wires in microns.
    pub diag_min_edge_length: Option<f64>,
    /// Minimum area for shapes on this layer.
    pub min_area: Option<f64>,
    /// Minimal rectangles that must fit in each shape on this layer.
    /// At least one needs to fit for each shape.
    /// Tuples of `(minimal width, minimal length)`.
    pub min_size: Vec<(f64, f64)>,
    /// Minimal edge length for shapes.
    pub min_step: (),
    /// Spacing rules.
    pub spacing: Vec<LefSpacingRules>,
    /// Spacing tables for spacing between wires.
    pub spacing_table: Option<LefSpacingTable>,
    /// Length of extension of a wire over a via. The extension must be at least half of the
    /// wire width.
    pub wire_extension: Option<f64>,
    /// Minimal number of cuts of a via depending on the width of the wire.
    pub minimum_cut: (),
    /// Maximum wire width in microns.
    pub max_width: Option<f64>,
    /// Minimum wire width in microns.
    pub min_width: Option<f64>,
    /// Minimum area of holes in metal shapes.
    /// `(min area [um^2], width [um])` tuples.
    /// If a width is specified the rule only applies if at least one of the wires around the hole
    /// has a larger width.
    pub min_enclosed_area: Vec<(f64, Option<f64>)>,
    /// Width of a protrusion.
    pub protrusion_width: (),
    /// Sheet resistance `[Ohm/square]`.
    pub resistance: Option<f64>,
    /// Specify wire-to-ground capacitance per square unit in `[pF/um^2]`.
    pub capacitance: Option<f64>,
    /// Distance from top of ground plane to bottom of this interconnect layer.
    pub height: Option<f64>,
    /// Thickness of the layer in microns.
    pub thickness: Option<f64>,
    /// Amount of loss in width of wires caused by the etching process.
    pub shrinkage: Option<f64>,
    /// Account for increase in capacitance caused by close wires.
    /// Default is 1.
    pub cap_multiplier: u32,
    /// `[pF/um]`.
    pub edge_capacitance: Option<f64>,
    /// Maximum allowed metal density in percent.
    pub minimum_density: Option<f64>,
    /// Minimum allowed metal density in percent.
    pub maximum_density: Option<f64>,
    /// Length and width of the density check window.
    pub density_check_window: Option<(f64, f64)>,
    /// Stepping distance for metal density checks.
    pub density_check_step: Option<f64>,
    /// Spacing between metal fills and active geometries.
    pub fill_active_spacing: Option<f64>,

    /// Antenna rule definitions.
    pub antenna_rules: LefAntennaRules,

    /// AC current density information.
    pub ac_current_density: Option<()>,
    /// Average DC current density information.
    /// Stored as a `(wire width, current density)` table.
    /// If only a default value is specified for all widths
    /// it is stored as a single entry for wire width `0`: `(0, default_current_density)`.
    /// Unit: `[mA/um]`
    pub dc_current_density: Vec<(f64, f64)>,

    /// Custom properties.
    pub properties: HashMap<String, LefPropertyValue>,
}

impl Default for LefRoutingLayer {
    /// Custom implementation of the `Default` trait for `RoutingLayer.
    fn default() -> Self {
        Self {
            name: Default::default(),
            mask_num: Default::default(),
            direction: LefRoutingDirection::Vertical,
            pitch: (0.0, 0.0),
            diag_pitch: Default::default(),
            width: 0.0,
            offset: Default::default(),
            diag_width: Default::default(),
            diag_spacing: Default::default(),
            diag_min_edge_length: Default::default(),
            min_area: Default::default(),
            min_size: Default::default(),
            min_step: Default::default(),
            spacing: Default::default(),
            spacing_table: Default::default(),
            wire_extension: Default::default(),
            minimum_cut: Default::default(),
            max_width: Default::default(),
            min_width: Default::default(),
            min_enclosed_area: Default::default(),
            protrusion_width: Default::default(),
            resistance: Default::default(),
            capacitance: Default::default(),
            height: Default::default(),
            thickness: Default::default(),
            shrinkage: Default::default(),
            // Take multiplicative identity as default for the capacitance multiplier.
            cap_multiplier: 1,
            edge_capacitance: Default::default(),
            minimum_density: Default::default(),
            maximum_density: Default::default(),
            density_check_window: Default::default(),
            density_check_step: Default::default(),
            fill_active_spacing: Default::default(),
            antenna_rules: LefAntennaRules::default(),
            ac_current_density: Default::default(),
            dc_current_density: Default::default(),
            properties: Default::default(),
        }
    }
}

///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LefMacroClass {
    /// Macro with fixed position.
    /// Commonly used for power routing. COVER does not contain active devices.
    /// A COVER class can have the sub-class BUMP. Typically BUMP cells have
    /// geometries only on the topmost 'bump' layer.
    COVER(bool),
    /// Big macro with internal power mesh.
    RING,
    /// Predefined macro.
    BLOCK(Option<LefMacroClassBlockType>),
    /// I/O pad.
    PAD(Option<LefMacroClassPadType>),
    /// Standard-cell macro used inside the core area.
    CORE(Option<LefMacroClassCoreType>),
    /// Start or end of core rows. Typically used to connect to the power grid.
    ENDCAP(Option<LefMacroClassEndcapType>),
}

impl FromStr for LefMacroClass {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "COVER" => Ok(Self::COVER(false)),
            "RING" => Ok(Self::RING),
            "BLOCK" => Ok(Self::BLOCK(Default::default())),
            "PAD" => Ok(Self::PAD(Default::default())),
            "CORE" => Ok(Self::CORE(Default::default())),
            "ENDCAP" => Ok(Self::ENDCAP(Default::default())),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefMacroClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::COVER(bump) => {
                f.write_str("COVER")?;
                if *bump {
                    f.write_str("BUMP")?;
                }
            }
            Self::RING => f.write_str("RING")?,
            Self::BLOCK(sub_class) => {
                f.write_str("BLOCK")?;
                if let Some(sub_class) = sub_class {
                    sub_class.fmt(f)?;
                }
            }
            Self::PAD(sub_class) => {
                f.write_str("PAD")?;
                if let Some(sub_class) = sub_class {
                    sub_class.fmt(f)?;
                }
            }
            Self::CORE(sub_class) => {
                f.write_str("CORE")?;
                if let Some(sub_class) = sub_class {
                    sub_class.fmt(f)?;
                }
            }
            Self::ENDCAP(sub_class) => {
                f.write_str("ENDCAP")?;
                if let Some(sub_class) = sub_class {
                    sub_class.fmt(f)?;
                }
            }
        }
        Ok(())
    }
}

/// Specify the type of a site: Either IO site (PAD) or core site (CORE).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LefSiteClass {
    /// A core site.
    CORE,
    /// An IO site.
    PAD,
}

impl Default for LefSiteClass {
    fn default() -> Self {
        Self::CORE
    }
}

impl FromStr for LefSiteClass {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "CORE" => Ok(Self::CORE),
            "PAD" => Ok(Self::PAD),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefSiteClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CORE => f.write_str("CORE")?,
            Self::PAD => f.write_str("PAD")?,
        };
        Ok(())
    }
}

/// Subclass of the BLOCK macro class.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LefMacroClassBlockType {
    /// A block which may only contain a SIZE statements for size estimation.
    /// A blackbox block is missing the implementation of the sub-block.
    BLACKBOX,
    /// A cell with partial implementation of the sub-block.
    SOFT,
}

impl FromStr for LefMacroClassBlockType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "BLACKBOX" => Ok(Self::BLACKBOX),
            "SOFT" => Ok(Self::SOFT),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefMacroClassBlockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BLACKBOX => f.write_str("BLACKBOX"),
            Self::SOFT => f.write_str("SOFT"),
        }
    }
}

/// Subclass of the PAD macro class.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LefMacroClassPadType {
    /// Input pad.
    INPUT,
    /// Output pad.
    OUTPUT,
    /// Inout pad.
    INOUT,
    /// Power pad.
    POWER,
    /// Spacer in the pad ring.
    SPACER,
    /// Area for I/O drivers with out connection to a bump. They need routing to a
    /// CLASS COVER BUMP macro for proper connection with the IC package.
    AREAIO,
}

impl FromStr for LefMacroClassPadType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "INPUT" => Ok(Self::INPUT),
            "OUTPUT" => Ok(Self::OUTPUT),
            "INOUT" => Ok(Self::INOUT),
            "POWER" => Ok(Self::POWER),
            "SPACER" => Ok(Self::SPACER),
            "AREAIO" => Ok(Self::AREAIO),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefMacroClassPadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::INPUT => f.write_str("INPUT"),
            Self::OUTPUT => f.write_str("OUTPUT"),
            Self::INOUT => f.write_str("INOUT"),
            Self::POWER => f.write_str("POWER"),
            Self::SPACER => f.write_str("SPACER"),
            Self::AREAIO => f.write_str("AREAIO"),
        }
    }
}

/// Subclass of the CORE macro class.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LefMacroClassCoreType {
    /// Connect to another cell.
    FEEDTHRU,
    /// Logical one.
    TIEHIGH,
    /// Logical zero.
    TIELOW,
    /// Spacer/fill cell.
    SPACER,
    /// Antenna diode.
    ANTENNACELL,
    /// Well-tap cell.
    WELLTAP,
}

impl FromStr for LefMacroClassCoreType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "FEEDTHRU" => Ok(Self::FEEDTHRU),
            "TIEHIGH" => Ok(Self::TIEHIGH),
            "TIELOW" => Ok(Self::TIELOW),
            "SPACER" => Ok(Self::SPACER),
            "ANTENNACELL" => Ok(Self::ANTENNACELL),
            "WELLTAP" => Ok(Self::WELLTAP),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefMacroClassCoreType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FEEDTHRU => f.write_str("FEEDTHRU"),
            Self::TIEHIGH => f.write_str("TIEHIGH"),
            Self::TIELOW => f.write_str("TIELOW"),
            Self::SPACER => f.write_str("SPACER"),
            Self::ANTENNACELL => f.write_str("ANTENNACELL"),
            Self::WELLTAP => f.write_str("WELLTAP"),
        }
    }
}

/// Subclass of the ENDCAP macro class.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LefMacroClassEndcapType {
    /// Start of the row (left).
    PRE,
    /// End of the row (right)
    POST,
    /// I/O corner cell on top left.
    TOPLEFT,
    /// I/O corner cell on top right.
    TOPRIGHT,
    /// I/O corner cell on bottom left.
    BOTTOMLEFT,
    /// I/O corner cell on bottom right.
    BOTTOMRIGHT,
}

impl FromStr for LefMacroClassEndcapType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "PRE" => Ok(Self::PRE),
            "POST" => Ok(Self::POST),
            "TOPLEFT" => Ok(Self::TOPLEFT),
            "TOPRIGHT" => Ok(Self::TOPRIGHT),
            "BOTTOMLEFT" => Ok(Self::BOTTOMLEFT),
            "BOTTOMRIGHT" => Ok(Self::BOTTOMRIGHT),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefMacroClassEndcapType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PRE => f.write_str("PRE"),
            Self::POST => f.write_str("POST"),
            Self::TOPLEFT => f.write_str("TOPLEFT"),
            Self::TOPRIGHT => f.write_str("TOPRIGHT"),
            Self::BOTTOMLEFT => f.write_str("BOTTOMLEFT"),
            Self::BOTTOMRIGHT => f.write_str("BOTTOMRIGHT"),
        }
    }
}

/// Data type of a property value.
#[derive(Clone, Debug)]
pub enum LefPropertyType {
    /// Integer number.
    Integer,
    /// Floating point number.
    Real,
    /// String.
    String,
}

impl FromStr for LefPropertyType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "INTEGER" => Ok(Self::Integer),
            "REAL" => Ok(Self::Real),
            "STRING" => Ok(Self::String),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefPropertyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer => f.write_str("INTEGER"),
            Self::Real => f.write_str("REAL"),
            Self::String => f.write_str("STRING"),
        }
    }
}

/// Value of a LEF/DEF property.
#[derive(Clone, Debug)]
pub enum LefPropertyValue {
    /// Integer.
    Int(i32),
    /// Floating point number.
    Real(f64),
    /// Quoted ASCII string.
    String(String),
}

impl fmt::Display for LefPropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LefPropertyValue::Int(v) => write!(f, "{}", v),
            LefPropertyValue::Real(v) => write!(f, "{}", v),
            LefPropertyValue::String(v) => write!(f, r#""{}""#, v),
        }
    }
}

/// Macro orientations that can be used by the placer.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct LefSymmetry {
    /// Mirroring macro at x-axis.
    pub x: bool,
    /// Mirroring macro at y-axis.
    pub y: bool,
    /// Rotating by 90 degrees. Intended for pad cells only.
    pub r90: bool,
}

impl LefSymmetry {
    /// Create a new symmetry definition.
    pub fn new(x: bool, y: bool, r90: bool) -> Self {
        Self { x, y, r90 }
    }

    /// Mirror symmetry at x-axis.
    pub fn x() -> Self {
        Self::new(true, false, false)
    }
    /// Mirror symmetry at y-axis.
    pub fn y() -> Self {
        Self::new(false, true, false)
    }
    /// Rotation by 90 degrees.
    pub fn r90() -> Self {
        Self::new(false, false, true)
    }

    /// Take the union of the both symmetry definitions.
    pub fn union(self, other: Self) -> Self {
        Self {
            x: self.x | other.x,
            y: self.y | other.y,
            r90: self.r90 | other.r90,
        }
    }
}

impl FromStr for LefSymmetry {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "X" => Ok(Self::x()),
            "Y" => Ok(Self::y()),
            "R90" => Ok(Self::r90()),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefSymmetry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.x {
            f.write_str("X")?;
        }
        if self.y {
            f.write_str("Y")?;
        }
        if self.r90 {
            f.write_str("R90")?;
        }

        Ok(())
    }
}

/// Antenna rule definitions.
/// TODO: 
#[derive(Clone, Debug, Default)]
pub struct LefAntennaRules {
}

/// Orientation, consists of rotation and mirroring.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LefOrient {
    /// North.
    N,
    /// South.
    S,
    /// East.
    E,
    /// West.
    W,
    /// Flipped North.
    FN,
    /// Flipped South.
    FS,
    /// FLipped East.
    FE,
    /// Flipped West.
    FW,
}

impl Default for LefOrient {
    fn default() -> Self {
        Self::N
    }
}

impl LefOrient {
    /// Decompose into a non-flipped orientation and a flag telling whether the orientation is flipped
    /// or not.
    /// Flipping is applied after rotation. Flips happen about the y-axis (switching left-right).
    ///
    /// Returns `(orientation, is_flipped)`.
    pub fn decomposed(&self) -> (Self, bool) {
        match self {
            LefOrient::FN => (LefOrient::N, true),
            LefOrient::FS => (LefOrient::S, true),
            LefOrient::FE => (LefOrient::E, true),
            LefOrient::FW => (LefOrient::W, true),
            other => (*other, false),
        }
    }

    /// Returns the flipped orientation.
    /// For example turns a `N` into a `FN`.
    pub fn flipped(&self) -> Self {
        use LefOrient::*;
        match self {
            N => FN,
            S => FS,
            E => FE,
            W => FW,
            FN => N,
            FS => S,
            FE => E,
            FW => W,
        }
    }
}

impl FromStr for LefOrient {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "N" => Ok(Self::N),
            "S" => Ok(Self::S),
            "E" => Ok(Self::E),
            "W" => Ok(Self::W),
            "FN" => Ok(Self::FN),
            "FS" => Ok(Self::FS),
            "FE" => Ok(Self::FE),
            "FW" => Ok(Self::FW),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefOrient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::N => f.write_str("N"),
            Self::S => f.write_str("S"),
            Self::E => f.write_str("E"),
            Self::W => f.write_str("W"),
            Self::FN => f.write_str("FN"),
            Self::FS => f.write_str("FS"),
            Self::FE => f.write_str("FE"),
            Self::FW => f.write_str("FW"),
        }
    }
}

/// Signal direction of a pin.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LefPinDirection {
    /// INPUT
    Input,
    /// OUTPUT. Is TRISTATE when the boolean flag is set.
    Output(bool),
    /// INOUT: Both input and output.
    Inout,
    /// FEEDTHRU: Pin crosses the cell. Direct electrical connection.
    Feedthru,
}

impl FromStr for LefPinDirection {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "INPUT" => Ok(Self::Input),
            "OUTPUT" => Ok(Self::Output(false)),
            "INOUT" => Ok(Self::Inout),
            "FEEDTHRU" => Ok(Self::Feedthru),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LefPinDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => f.write_str("INPUT"),
            Self::Output(false) => f.write_str("OUTPUT"),
            Self::Output(true) => f.write_str("OUTPUT TRISTATE"),
            Self::Inout => f.write_str("INOUT"),
            Self::Feedthru => f.write_str("FEEDTHRU"),
        }
    }
}
