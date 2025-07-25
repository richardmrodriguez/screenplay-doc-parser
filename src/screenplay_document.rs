use crate::{pdf_document, };
use core::panic;
use std::{
    collections::{HashMap, HashSet,},
    
    hash::Hash,
    ops::{Deref, DerefMut, },
    time::{Instant, },
    vec,
};
use uuid::Uuid;

#[derive(PartialEq, Clone, Debug)]
pub enum TimeOfDay {
    Day(String),
    Night(String),
    Morning(String),
    Evening(String),
    Afternoon(String),
    Extras(Option<HashMap<String, String>>),
}


#[derive(PartialEq, Clone, Debug)]
pub struct TimeOfDayCollection {
    pub day: TimeOfDay,
    pub night: TimeOfDay,
    pub morning: TimeOfDay,
    pub evening: TimeOfDay,
    pub afternoon: TimeOfDay,
    pub extras: Option<HashMap<String, String>>,
}
impl Default for TimeOfDayCollection {
    fn default() -> Self {
        return Self {
            day: TimeOfDay::Day("DAY".into()),
            night: TimeOfDay::Night("NIGHT".into()),
            morning: TimeOfDay::Morning("MORNING".into()),
            evening: TimeOfDay::Evening("EVENING".into()),
            afternoon: TimeOfDay::Afternoon("AFTERNOON".into()),
            extras: None,
        };
    }
}
impl TimeOfDayCollection {
    pub fn is_time_of_day(&self, target: &String) -> bool {
        let vars: Vec<&TimeOfDay> = vec![
            &self.day,
            &self.night,
            &self.morning,
            &self.evening,
            &self.afternoon,
        ];

        for time in vars {
            match time {
                TimeOfDay::Day(string)
                | TimeOfDay::Night(string)
                | TimeOfDay::Morning(string)
                | TimeOfDay::Evening(string)
                | TimeOfDay::Afternoon(string) => {
                    if string == target {
                        return true;
                    }
                }
                _ => {}
            }
        }

        match &self.extras {
            None => {
                return false;
            }
            Some(e) => {
                for (_, string) in e {
                    if target == string {
                        return true;
                    }
                }
            }
        }

        return false;
    }

    pub fn get_time_of_day(&self, target: &String) -> Option<TimeOfDay> {
        let vars: Vec<&TimeOfDay> = vec![
            &self.day,
            &self.night,
            &self.morning,
            &self.evening,
            &self.afternoon,
        ];

        for time in vars {
            match time {
                TimeOfDay::Day(string)
                | TimeOfDay::Night(string)
                | TimeOfDay::Morning(string)
                | TimeOfDay::Evening(string)
                | TimeOfDay::Afternoon(string) => {
                    if string == target {
                        return Some(time.clone());
                    }
                }
                _ => {}
            }
        }

        match &self.extras {
            None => {
                return None;
            }
            Some(e) => {
                for (_, string) in e {
                    if target == string {
                        return Some(TimeOfDay::Extras(None)); // this is FUCKING horrendous what the fuck man
                    }
                }
            }
        }

        return None;
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PageFormat {
    US,
    A4,
    OTHER,
}

/// # SPType
///
/// The various Element Types found in a Screenplay.
///
/// Types can be assigned to both individual `TextElements` and `Lines`.
///
/// Some `Line`s will only contain a single type.
///
/// An `SPType::SP_ACTION` Line will contain only `SPType::SP_ACTION` text elements, for example.
///
/// But a `SP_CHARACTER` line will potentially contain one or more `SP_CHARACTER` elements, as well as one or more `SP_CHARACTER_EXTENSION` elements:
///
/// ```text
/// ...
///
///         CHARLIE (V.O.)
///     I always wanted to be a gangster.
///
/// ...
///
/// ```
///
/// Notice how `CHARLIE` is the `SP_CHARACTER` and the `(V.O.)` is the `SP_CHARACTER_EXTENSION`.
///
/// ## Scene Headings
///
/// A Scene Heading will consist of multiple types, such as the `SP_ENVIRONMENT`, meaning interior and/or exterior (INT./EXT.), the location, sublocation, and time of day:
///
/// `EXT. BASEBALL FIELD - PITCHER'S MOUND - DAY`
///
/// Scene headings can contain more element types, such as a Time Period, or multiple Sublocations.
#[derive(Default, Clone, Debug, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum SPType {
    SP_ACTION = 0,

    SP_CHARACTER,
    SP_CHARACTER_EXTENSION, // require context to parse (previous word type)
    SP_DG_MORE_CONTINUED,   // specifically has MORE or CONTINUED or CONT'D within parentheses
    SP_PARENTHETICAL,
    SP_DIALOGUE,
    SP_TRANSITION,

    /// SCENE HEADING
    ///
    SP_SCENE_HEADING(SceneHeadingElement), // begins with INT. , EXT. , or I./E.

    /// `INT.`, `EXT.`, `INT./EXT.`, etc.
    //SP_ENVIRONMENT,
    //SP_LOCATION,
    //SP_SCENE_HEADING_SUB_ELEMENT,
    //SP_SCENE_HEADING_SEPARATOR, /// Breaks up a slugline -- EXT. BASEBALL FIELD - PITCHER'S MOUND - PAST - NIGHT
    //SP_SCENE_TIMEPERIOD, // PAST, PRESENT, FUTURE, arbitrary timeframe "BEFORE DINNER", "AFTER THE EXPLOSION", etc.
    //SP_SUBLOCATION,
    //SP_TIME_OF_DAY,
    SP_SHOT_ANGLE, // SHOT or ANGLE on something, NOT a full scene heading / location

    SP_PAGENUM,  // Nominal page number
    SP_SCENENUM, // Nominal scene number

    SP_PAGE_HEADER, //LINE --contains the PAGE NUM and potentially a page Revision label
    SP_PAGE_REVISION_LABEL, //may or may not include the date / color (I think it's two lines usually, but it could be one line potentially...?)
    SP_LINE_REVISION_MARKER, // asterisks in the left and/or right margins indicate a line or lines have been revised

    SP_MORE_CONTINUED,
    SP_FOOTER, // Not sure what footers are used for but....

    //DUAL DIALOGUE
    SP_DUAL_CHARACTERS,
    SP_DUAL_DIALOGUES,

    SP_DD_L_CHARACTER,
    SP_DD_L_CHARACTER_EXTENSION,
    SP_DD_L_PARENTHETICAL,
    SP_DD_L_DIALOGUE,
    SP_DD_L_MORE_CONTINUED,

    SP_DD_R_CHARACTER,
    SP_DD_R_CHARACTER_EXTENSION,
    SP_DD_R_PARENTHETICAL,
    SP_DD_R_DIALOGUE,
    SP_DD_R_MORE_CONTINUED,

    // TITLE PAGE
    TP_TITLE,
    TP_BYLINE,
    TP_AUTHOR,
    TP_DRAFT_DATE,
    TP_CONTACT,
    // -------------
    SP_OTHER,
    SP_BLANK, // BLANK element?
    SP_OMITTED,
    // Non- content text (asterisks and/or scene numbers in the margins, headers and footers, page numbers, etc.)
    NON_CONTENT_TOP,
    NON_CONTENT_BOTTOM,
    NON_CONTENT_LEFT,
    NON_CONTENT_RIGHT,

    #[default]
    NONE,
    _TYPECOUNT,
}

// -------- SCREENPLAY TYPED STRUCTS / ENUMS

// -------------------- CHARACTER
#[derive(Default, PartialEq, Clone, Debug, Eq, Hash)]
pub struct CharacterID(Uuid);
impl Deref for CharacterID {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for CharacterID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl CharacterID {
    pub fn new() -> Self {
        CharacterID(Uuid::new_v4())
    }
}

#[derive(Default, PartialEq, Clone, Debug, Eq, Hash)]
pub struct Character {
    pub name: String,
    pub id: CharacterID,
}
impl Character {
    pub fn is_line(&self, line: &Line) -> bool {
        let mut maybe_character_name = String::new();
        let mut previous_type: Option<SPType> = Some(SPType::NONE);
        for text_element in &line.text_elements {
            if previous_type != text_element.element_type {
                if maybe_character_name == self.name {
                    println!("'howdy y'all");
                    return true;
                }
                maybe_character_name = String::new();
            }
            match text_element.element_type {
                Some(SPType::SP_CHARACTER)
                | Some(SPType::SP_DD_L_CHARACTER)
                | Some(SPType::SP_DD_R_CHARACTER) => {
                    if !maybe_character_name.is_empty() {
                        maybe_character_name.push(' ');
                    }
                    maybe_character_name.push_str(&text_element.text.clone());
                }
                _ => {}
            }
            previous_type = text_element.element_type.clone();
        }
        if maybe_character_name == self.name {
            //println!("HOO WEE!");
            return true;
        }

        false
    }
}

// -------------------- PAGE
#[derive(Default, PartialEq, Clone, Debug, Eq, Hash)]
pub struct PageID(pub Uuid);
impl Deref for PageID {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PageID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct PageNumber(pub String);
impl Deref for PageNumber {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for PageNumber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// -------------------- SCENE
#[derive(Default, PartialEq, Clone, Debug)]
pub struct SceneNumber(pub String);

#[derive(Default, PartialEq, Clone, Copy, Debug, Hash, Eq)]
pub struct SceneID(pub Uuid);
impl Deref for SceneID {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for SceneID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl SceneID {
    pub fn new() -> Self {
        SceneID(Uuid::new_v4())
    }
}

//TODO:
// make the SP_SCENE_HEADING element take one of THESE as data,
// instead of having the scene elements flattened out among the SP_TYPEs
// maybe also do this technique with CHARACTER, DIALOGUE, etc. ,
// basically make each element have the LINE TYPE, which contains the ELEMENT TYPE as data...
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum SceneHeadingElement {
    Line, // The Line Itself
    Environment,
    Location,
    SubLocation,
    TimeOfDay,
    Continuity, // CONTINUOUS
    TimePeriod, // EALIER, LATER, 1950s, WEDNESDAY, etc.
    Separator,  // hyphen
    SceneNumber,
    SlugOther,
}

//TODO: add get_scene_from_id func to ScreenplayDocument struct
#[derive(PartialEq, Clone, Debug)]
pub struct Scene {
    pub start: ScreenplayCoordinate,

    pub environment: Environment,
    pub number: Option<SceneNumber>,
    pub revised: bool,

    pub story_locations: Vec<LocationID>,
    pub story_time_of_day: Option<TimeOfDay>, // DAY, NIGHT, etc.
}

pub struct EnvironmentStrings {
    pub int: Vec<String>,
    pub ext: Vec<String>,
    pub combo: Vec<String>,
}
impl Default for EnvironmentStrings {
    fn default() -> Self {
        EnvironmentStrings {
            int: vec!["INT.".into()],
            ext: vec!["EXT.".into()],
            combo: vec![
                "INT./EXT.".into(),
                "I./E.".into(),
                "EXT./INT.".into(),
                "E./I.".into(),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Environment {
    Int,
    Ext,
    Combo(Option<Vec<Environment>>),
}
impl Environment {
    pub fn from_str(string: &String, current_env_strs: &EnvironmentStrings) -> Option<Self> {
        if current_env_strs.int.contains(&string) {
            return Some(Environment::Int);
        }
        if current_env_strs.ext.contains(&string) {
            return Some(Environment::Ext);
        }
        if current_env_strs.combo.contains(&string) {
            // TODO: actually hanndle combos (4 total possibilities, int/int, int/ext, ext/int, and ext/ext)
            return Some(Environment::Combo(None));
        }
        None
    }
}

#[derive(Default, PartialEq, Clone, Debug, Eq, Hash)]
pub struct LocationID(Uuid);

impl Deref for LocationID {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for LocationID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl LocationID {
    pub fn new() -> Self {
        LocationID(Uuid::new_v4())
    }
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct LocationNode {
    pub string: String,
    pub sublocations: HashSet<LocationID>, // list of IDs for other locations
    pub superlocation: Option<LocationID>, //
}
impl LocationNode {
    pub fn add_sublocation(&mut self, new_id: LocationID) -> bool {
        self.sublocations.insert(new_id)
    }

    ///
    /// Determines if a path exists under this LocationNode.
    ///
    /// ```
    /// let mut screenplay_doc = screenplay_document::ScreenplayDocument::new();
    /// ```
    ///
    pub fn subpath_exists<'a>(
        &'a self,
        this_location_id: &'a LocationID,
        subpath: &[String],
        screenplay: &'a ScreenplayDocument,
    ) -> Option<(&'a LocationID, Vec<String>)> {
        if subpath.is_empty() {
            return None;
        }

        let subpath_root = &subpath[0];

        for id in &self.sublocations {
            let Some(sublocation) = screenplay.locations.get(id) else {
                continue;
            };
            if sublocation.string == *subpath_root {
                if subpath.len() == 1 {
                    return Some((id, Vec::new()));
                }
                if subpath.len() > 1 && sublocation.sublocations.is_empty() {
                    return Some((id, Vec::from(&subpath[1..])));
                }
                return sublocation.subpath_exists(id, &subpath[1..], screenplay);
            }
        }

        Some((this_location_id, subpath.to_vec()))
    }
}

// --------------- BASIC DOCUMENT COMPONENTS ---------------

#[derive(Default, PartialEq, Clone, Debug)]
pub struct TextElement {
    pub text: String,
    pub element_type: Option<SPType>,
    pub preceding_whitespace_chars: u64,
    pub element_position: Option<pdf_document::TextPosition>,
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct Line {
    pub text_elements: Vec<TextElement>,
    pub scene_number: Option<String>,
    pub scene_id: Option<SceneID>,
    pub line_type: Option<SPType>,
    pub preceding_empty_lines: u64,
    pub revised: bool,
    pub blank: bool,
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct Page {
    pub lines: Vec<Line>,
    pub page_number: Option<PageNumber>,
    pub revised: bool,
    pub revision_label: Option<String>,
    pub revision_date: Option<String>,
    pub page_format: Option<PageFormat>,
}

#[derive(Default, PartialEq, Clone, Debug, Hash, Eq, PartialOrd)]
pub struct ScreenplayCoordinate {
    pub page: usize,
    pub line: usize,
    pub element: Option<u64>,
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct ScreenplayDocument {
    pub pages: Vec<Page>,
    pub revisions: Option<Vec<String>>, // current (and possible previous) revision date(s) from the title page
    pub scenes: HashMap<SceneID, Scene>,
    pub locations: HashMap<LocationID, LocationNode>,
    pub characters: HashSet<Character>,
    pub page_numbers: HashMap<PageID, PageNumber>,
}
impl ScreenplayDocument {
    pub fn new() -> Self {
        ScreenplayDocument {
            pages: Vec::new(),
            revisions: None,
            scenes: HashMap::new(),
            locations: HashMap::new(),
            characters: HashSet::new(),
            page_numbers: HashMap::new(),
        }
    }

    
}
