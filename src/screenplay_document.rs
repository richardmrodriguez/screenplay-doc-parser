use crate::{pdf_document, screenplay_document};
use core::panic;
use serde::de::IntoDeserializer;
use std::{
    collections::{HashMap, HashSet, btree_map::Keys},
    default,
    hash::Hash,
    ops::{Deref, DerefMut, Index},
    panic::Location,
    path::Component,
    thread::panicking,
    time::SystemTime,
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
impl TimeOfDay {
    fn get(&self) {}
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

// FIXME: TODO: Collapse the SPTypes, so that
// LINE TYPE variants take in specialized SUBTYPE enum variants
// I.E. each SCENE_HEADING TextElement will take in a SluglineElement as data
// each

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
    pub fn is_line(&self, line: &screenplay_document::Line) -> bool {
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
    pub fn check_if_subpath_exists(
        &self,
        this_location_id: &LocationID,
        subpath: &[String],
        screenplay: &screenplay_document::ScreenplayDocument,
    ) -> Option<(LocationID, Vec<String>)> {
        if subpath.is_empty() {
            return None;
        }

        let subpath_root = &subpath[0];

        for id in &self.sublocations {
            let Some(sublocation) = screenplay.get_location(&id) else {
                continue;
            };
            if sublocation.string == *subpath_root {
                if subpath.len() == 1 {
                    return Some((id.clone(), Vec::new()));
                }
                if subpath.len() > 1 && sublocation.sublocations.is_empty() {
                    return Some((id.clone(), Vec::from(&subpath[1..])));
                }
                return sublocation.check_if_subpath_exists(id, &subpath[1..], &screenplay);
            }
        }

        Some((this_location_id.clone(), subpath.to_vec()))
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

    ///
    /// Returns an Optional Vec of LocationIDs, up to and including this LocationID.
    ///
    pub fn get_full_location_path_for_node(
        &self,
        location: LocationID,
    ) -> Option<(Vec<LocationID>)> {
        unimplemented!();
        None
    }

    // ------------ Get LOCATIONs...

    ///
    /// Determines if a "location path" exists.
    ///
    /// Returns `None` if nothing matches the root of the path.
    ///
    /// Returns `Some((&LocationID, Vec<String>))` if a partial match is found.
    /// The `&LocationID` is the last valid location the path matched.
    /// The `Vec<String>` is the remaining subpath, which does not yet exist as `LocationNode`s.
    ///
    /// If the full path exists, the `Vec<String>` will be empty.
    ///
    /// The caller can afterwards handle creating the rest of the Location path,
    /// and appending a new LocationID to the sublocations field
    ///
    ///
    pub fn check_if_location_path_exists(
        &self,
        path: &[String],
    ) -> Option<(LocationID, Vec<String>)> {
        if path.is_empty() {
            return None;
        }

        let path_root = &path[0];

        for (id, location) in &self.locations {
            if location.string == *path_root {
                if path.len() == 1 {
                    return Some((id.clone(), Vec::new()));
                }
                if path.len() > 1 && location.sublocations.is_empty() {
                    return Some((id.clone(), Vec::from(&path[1..])));
                }

                return location.check_if_subpath_exists(id, &path[1..], &self);
            }
        }

        None
    }

    pub fn get_location(&self, id: &LocationID) -> Option<&LocationNode> {
        for (existing_id, loc) in &self.locations {
            if existing_id == id {
                return Some(loc);
            }
        }
        None
    }
    pub fn get_location_mutable(&mut self, id: &LocationID) -> Option<&mut LocationNode> {
        for (existing_id, loc) in &mut self.locations {
            if existing_id == id {
                return Some(loc);
            }
        }
        None
    }
    pub fn get_locations_with_matching_str(
        &self,
        location_string: &String,
    ) -> Option<Vec<&LocationID>> {
        let mut loc_id_vec: Vec<&LocationID> = Vec::new();
        for (id, location) in &self.locations {
            if location.string == *location_string {
                loc_id_vec.push(id);
            }
        }
        if loc_id_vec.is_empty() {
            return None;
        }
        Some(loc_id_vec)
    }

    // TODO: idk how to do this tree navigation shit without cloning IDs.......
    // there's most definitely a much better way to do this
    // maybe immutable reference counting?
    pub fn get_location_root(&self, location_id: &LocationID) -> Option<LocationID> {
        let Some(location) = self.get_location(&location_id) else {
            return None;
        };
        let Some(superlocation) = &location.superlocation else {
            return Some(location_id.clone());
        };
        return self.get_location_root(&superlocation.clone());
    }

    pub fn get_location_leafs(&self, location_id: &LocationID) -> Option<HashSet<LocationID>> {
        let Some(location) = self.get_location(&location_id) else {
            return None;
        };
        if location.sublocations.is_empty() {
            let mut this_leaf_as_set: HashSet<LocationID> = HashSet::new();
            this_leaf_as_set.insert(location_id.clone());
            return Some(this_leaf_as_set);
        }
        let mut location_id_set: HashSet<LocationID> = HashSet::new();
        for sublocation_id in &location.sublocations {
            let Some(subset) = self.get_location_leafs(&sublocation_id.clone()) else {
                continue;
            };
            location_id_set.extend(subset);
        }
        if location_id_set.is_empty() {
            return None;
        }
        Some(location_id_set)
    }

    pub fn get_locations_with_character(&self, character_name: String) -> Option<Vec<&LocationID>> {
        unimplemented!();
        let Some(ch_id) = self.get_character_id_from_name(&character_name) else {
            return None;
        };

        None
    }

    pub fn get_locations_for_page(&self, page: PageID) -> Option<Vec<&LocationID>> {
        unimplemented!();
        let scenes = self.get_scenes_on_page_by_page_id(&page)?;
        let locations = scenes
            .iter()
            .map(|s| self.get_scene_from_id(s))
            .filter(|scn| scn.is_some())
            .map(|scn| scn.unwrap())
            .map(|scn| scn.story_locations.clone())
            //.collect()
            ;
        None
    }

    // ------------ Get COORDINATEs...

    // ------------ Get LINEs...

    pub fn filter_lines_by_scene<'a>(
        &self,
        lines: &HashMap<ScreenplayCoordinate, &'a Line>,
        scenes_filter: Vec<&SceneID>,
    ) -> Option<HashMap<ScreenplayCoordinate, &'a Line>> {
        if scenes_filter.is_empty() {
            //panic!("EMPTY FILTER!");
            return None;
        }
        let mut scn_filtered: HashMap<ScreenplayCoordinate, &Line> = HashMap::new();

        lines
            .iter()
            .filter(|(coord, ln)| {
                let Some(scene_id) = self.get_scene_id_for_screenplay_coordinate(&coord) else {
                    
                    return false;
                };
                if scenes_filter.contains(&scene_id)  {
                    return  true;
                }
                false
            })
            .for_each(|(coord, line)| {
                scn_filtered.insert(coord.clone(), line);
            });

        if scn_filtered.is_empty() {
            return None;
        }
        if scn_filtered.len() == lines.len() {
            //panic!("DIDN'T FILTER ANY LINES AT ALL!!!");
        }
        Some(scn_filtered)
    }

    pub fn get_lines_of_dialogue_for_character(
        &self,
        character: &Character,
    ) -> Option<HashMap<ScreenplayCoordinate, &Line>> {
        let mut lines_with_coords: HashMap<ScreenplayCoordinate, &Line> = HashMap::new();
        //let mut lines: Vec<&Line> = Vec::new();
        let mut is_dialogue = false;

        for (p_index, page) in self.pages.iter().enumerate() {
            for (l_index, line) in page.lines.iter().enumerate() {
                if line.line_type != Some(SPType::SP_CHARACTER)
                    && line.line_type != Some(SPType::SP_DUAL_CHARACTERS)
                    && line.line_type != Some(SPType::SP_DIALOGUE)
                    && line.line_type != Some(SPType::SP_DUAL_DIALOGUES)
                {
                    continue;
                }
                //println!("MIGHT BE CHARACTER OR DIALOGUE");
                if character.is_line(line) {
                    //println!("Oh boy!!!! | {:?} | {:?}", line.line_type, line.text_elements);
                    is_dialogue = true;
                    continue;
                }
                if !is_dialogue {
                    continue;
                }
                match line.line_type {
                    Some(SPType::SP_DIALOGUE) | Some(SPType::SP_DUAL_DIALOGUES) => {
                        lines_with_coords.insert(
                            ScreenplayCoordinate {
                                page: p_index,
                                line: l_index,
                                element: None,
                            },
                            line,
                        );
                    }
                    _ => {
                        is_dialogue = false;
                        continue;
                    }
                }
            }
        }
        if lines_with_coords.is_empty() {
            return None;
        }
        Some(lines_with_coords)
    }

    pub fn get_line_from_coordinate(
        &self,
        coordinate: &ScreenplayCoordinate,
    ) -> Option<&screenplay_document::Line> {
        let Some(page) = self.pages.get(coordinate.page) else {
            return None;
        };
        let Some(line) = page.lines.get(coordinate.line) else {
            return None;
        };

        Some(line)
    }

    // ------------ Get CHARACTERS...

    pub fn get_character_id_from_name(&self, name: &String) -> Option<&CharacterID> {
        for ch in &self.characters {
            if &ch.name == name {
                return Some(&ch.id);
            }
        }
        None
    }

    pub fn get_character_from_name(&self, name: &String) -> Option<&Character> {
        for ch in &self.characters {
            if &ch.name == name {
                return Some(&ch);
            }
        }
        None
    }

    pub fn get_character_from_id(&self, id: &CharacterID) -> Option<&Character> {
        for ch in &self.characters {
            if &ch.id == id {
                return Some(&ch);
            }
        }
        None
    }

    pub fn get_characters_for_scene(&self, scene_id: &SceneID) -> Option<Vec<&CharacterID>> {
        None
    }

    pub fn get_characters_for_page(&self, page_num_id: &PageID) -> Option<Vec<&Character>> {
        None
    }

    // ------------ Get SCENES...

    pub fn filter_scenes_by_locations(
        &self,
        scene_ids: Vec<&SceneID>,
        locations: Vec<&LocationID>,
    ) -> Option<Vec<&SceneID>> {
        let mut filtered: Vec<&SceneID> = Vec::new();
        for loc in locations {
            let Some(scenes_for_loc) = self.get_scenes_with_location(loc) else {
                panic!("COULDN'T FIND SCENES WITH THIS LOCATION?!");
                continue;
            };

            for scene in scenes_for_loc {
                println!("----- FILTERING BY LOCATION-SCENE....");
                if scene_ids.contains(&scene) && !filtered.contains(&scene) {
                    filtered.push(scene);
                }
            }
        }
        if filtered.is_empty() {
            return None;
        }
        Some(filtered)
    }

    /// Gets a Vec of all `SceneID`s in the document, sorted by document order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use screenplay_doc_parser_rs::screenplay_document::*;
    /// let mut doc = ScreenplayDocument::new();
    /// let scene1: Scene = Scene {
    ///     start: ScreenplayCoordinate {page: 0 as usize, line: 10 as usize, element: None},
    ///     number: None,
    ///     revised: false,
    ///     environment: Environment::Int,
    ///     story_location: Location::default(),
    ///     story_sublocation: None,
    ///     story_time_of_day: None
    /// };
    /// let scene2: Scene = Scene {
    ///     start: ScreenplayCoordinate {page: 1 as usize, line: 5 as usize, element: None},
    ///     number: None,
    ///     revised: false,
    ///     environment: Environment::Int,
    ///     story_location: Location::default(),
    ///     story_sublocation: None,
    ///     story_time_of_day: None
    /// };
    /// let id_1 = SceneID::new();
    /// let id_2 = SceneID::new();
    /// doc.scenes.insert(id_2.clone(), scene2);
    /// doc.scenes.insert(id_1.clone(), scene1);
    ///
    /// let sorted = doc.get_all_scenes_sorted().unwrap();
    ///
    /// assert_eq!(**sorted.get(0).unwrap(), id_1);
    /// assert_eq!(**sorted.get(1).unwrap(), id_2)
    /// ```
    pub fn get_all_scenes_sorted(&self) -> Option<Vec<&SceneID>> {
        if self.scenes.len() == 0 {
            return None;
        }
        if self.scenes.len() == 1 {
            return Some(self.scenes.keys().collect());
        }
        let mut scene_ids: Vec<_> = self.scenes.keys().collect();

        scene_ids.sort_by(|a, b| {
            let scn_a = self.scenes.get(a).unwrap();
            let scn_b = self.scenes.get(b).unwrap();

            (scn_a.start.page, scn_a.start.line).cmp(&(scn_b.start.page, scn_b.start.line))
        });
        return Some(scene_ids);
    }

    pub fn get_scene_id_for_screenplay_coordinate(
        &self,
        screenplay_coordinate: &ScreenplayCoordinate,
    ) -> Option<&SceneID> {
        let Some(page) = self.pages.get(screenplay_coordinate.page) else {
            println!("DUCK SAUCE 2: HELP!");
            return None;
        };

        // lines are properly enumerated FORWARDS, then ENTIRE ITERATOR is reversed
        for (line_index, line) in page.lines.iter().enumerate().rev() {
            // if this line is EQUAL or EARLIER THAN the coordinate...
            if line_index <= screenplay_coordinate.line {
                if let Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line)) = line.line_type {
                    //print!("SUCCESS --- ");
                    // for te in &line.text_elements {
                    //     print!("{:?} ", te.text);
                    // }
                    //print!("| ");
                    //println!(
                    //    "COORD_LINE_IDX: {:?} | LINE INDEX: {:?} | PAGE_LINES_LEN: {:?}",
                    //    screenplay_coordinate.line,
                    //    line_index,
                    //    page.lines.len()
                    //);
                    let Some(scn_id) = &line.scene_id else {
                        //println!("{:?}", &line.scene_id);
                        //println!("SCENE HEADING HAS NO SCENE ID!");
                        //println!("CHECKING...");
                        
                        println!("SCENE HEADING HAS NO SCENE ID!");
                        panic!();
                        continue;
                    };
                    //println!("SUCCESS PART ONE...?");
                    return Some(scn_id);
                }
            }
        }
        // couldn't find the scene on this page. try the previous page...
        // recursively check all previous pages

        let Some(previous_page_idx) = screenplay_coordinate.page.checked_sub(1) else {
            return None;
        };
        let Some(previous_page) = self.pages.get(previous_page_idx) else {
            return None;
        };
        let Some(last_line_idx) = previous_page.lines.len().checked_sub(1) else {
            return None;
        };
        let last_page_last_line_coord = ScreenplayCoordinate {
            page: previous_page_idx,
            line: last_line_idx,
            element: None,
        };
        //println!("CHECKING RECURSIVELY FOR SCENE!");
        //println!("COORDS OF PREVIOUS_PAGE: {:?}", last_page_last_line_coord);
        
        let Some(id_opt) = self.get_scene_id_for_screenplay_coordinate(&last_page_last_line_coord)
        else {
            //println!("RECURSIVE CHECK FAILED!");
            //panic!("DUCK SAUCE");
            return None;
        };
        //println!("SUCCESS PART TWO...?");
        return Some(id_opt);
    }

    pub fn get_scene_ids_from_range(
        &self,
        start: &ScreenplayCoordinate,
        end: &ScreenplayCoordinate,
    ) -> Option<Vec<&SceneID>> {
        if (self.pages.get(start.page)).is_none() {
            println!("{:?}", start);
            panic!();
            return None;
        }
        if self.pages.get(end.page).is_none() {
            return None;
        }

        let mut scenes: Vec<&SceneID> = Vec::new();
        for page_index in start.page..=end.page {
            let Some(page) = self.pages.get(page_index) else {
                continue;
            };

            for (l_idx, line) in page.lines.iter().enumerate() {
                if page_index == start.page && l_idx < start.line {
                    continue;
                } else if page_index == end.page && l_idx > end.line {
                    break;
                }
                // TODO: keep track of CURRENT line type and PREVIOUS linetype
                // if PREV line was a SCENE HEADING, just continue
                // if CURRENT LINE is NOT a scene heading, AND we already HAVE an entry in the scenes vec, CONTINUE
                if Some(SPType::SP_SCENE_HEADING(SceneHeadingElement::Line)) != line.line_type
                    && !scenes.is_empty()
                {
                    continue;
                }
                let Some(scene_id) =
                    self.get_scene_id_for_screenplay_coordinate(&ScreenplayCoordinate {
                        page: page_index,
                        line: l_idx,
                        element: None,
                    })
                else {
                    continue;
                };
                if !scenes.contains(&scene_id) {
                    scenes.push(scene_id);
                }
            }
        }
        if scenes.is_empty() {
            return None;
        }

        Some(scenes)
    }

    pub fn get_scene_from_id(&self, id: &SceneID) -> Option<&Scene> {
        let scene = self.scenes.get(id)?;
        Some(scene)
    }
    pub fn get_scenes_from_ids(&self, ids: Vec<&SceneID>) -> Option<Vec<&Scene>> {
        let mut scenes: Vec<&Scene> = Vec::new();
        for id in ids {
            let scene = self.scenes.get(id)?;
            scenes.push(scene);
        }
        if scenes.is_empty() {
            return None;
        }

        Some(scenes)
    }

    pub fn get_scenes_with_scene_heading_element(
        &self,
        heading_element: &screenplay_document::SceneHeadingElement,
    ) -> Option<Vec<&SceneID>> {
        let mut scene_ids: Vec<&SceneID> = Vec::new();

        for (scene_id, scene) in &self.scenes {
            let Some(scene_heading_line) = self.get_line_from_coordinate(&scene.start) else {
                continue;
            };
            for element in &scene_heading_line.text_elements {
                let Some(eltype) = &element.element_type else {
                    continue;
                };
                let SPType::SP_SCENE_HEADING(sch) = eltype else {
                    continue;
                };

                if sch == heading_element {
                    scene_ids.push(scene_id);
                }
            }
        }

        if scene_ids.is_empty() {
            return None;
        }
        Some(scene_ids)
    }

    pub fn get_all_scenes_on_page(&self, page_index: usize) -> Option<Vec<&SceneID>> {
        let page = self.pages.get(page_index)?;
        let last_line_idx = page.lines.len().checked_sub(1)?;
        let start = ScreenplayCoordinate {
            page: page_index,
            line: 0,
            element: None,
        };
        let end = ScreenplayCoordinate {
            page: page_index,
            line: last_line_idx,
            element: None,
        };
        let scenes = self.get_scene_ids_from_range(&start, &end)?;

        Some(scenes)
    }

    pub fn get_scenes_with_character_speaking(
        &self,
        character: &Character,
    ) -> Option<Vec<&SceneID>> {
        let mut scenes_vec: Vec<&SceneID> = Vec::new();

        for (p_idx, page) in self.pages.iter().enumerate() {
            for (l_idx, line) in page.lines.iter().enumerate() {
                if character.is_line(&line) {
                    let scoord = ScreenplayCoordinate {
                        page: p_idx,
                        line: l_idx,
                        element: None,
                    };
                    let Some(sceneid) = self.get_scene_id_for_screenplay_coordinate(&scoord) else {
                        continue;
                    };
                    if !scenes_vec.contains(&sceneid) {
                        scenes_vec.push(sceneid);
                    }
                }
            }
        }
        if scenes_vec.is_empty() {
            return None;
        } else {
            return Some(scenes_vec);
        }

        let Some(scene_ids_in_order) = self.get_all_scenes_sorted() else {
            return None;
        };
        let mut scenes_with_char_speaking: Vec<&SceneID> = Vec::new();
        let Some(lines_of_dialogue_for_character) =
            self.get_lines_of_dialogue_for_character(character)
        else {
            return None;
        };

        let scenes_in_order: Vec<&Scene> = scene_ids_in_order
            .iter()
            .filter_map(|id| self.get_scene_from_id(id))
            .collect();

        let mut scenes_map: HashMap<&SceneID, &Scene> = HashMap::new();
        if scene_ids_in_order.len() != scenes_in_order.len() {
            return None;
        }
        for (counter, scene_id) in scene_ids_in_order.iter().enumerate() {
            scenes_map.insert(*scene_id, scenes_in_order[counter]);
        }

        let mut current_scene_id: &SceneID = scene_ids_in_order[0];
        let mut current_scene_coordinate: &ScreenplayCoordinate = &ScreenplayCoordinate {
            page: 0,
            line: 0,
            element: None,
        };

        'lines: for (coord, line) in &lines_of_dialogue_for_character {
            if coord.page < current_scene_coordinate.page {
                continue;
            }
            if coord.line < current_scene_coordinate.line {
                continue;
            }

            'scenes: for (rev_idx, scene_id) in scene_ids_in_order.iter().rev().enumerate() {
                if scenes_map[scene_id].start.page > coord.page
                    && scenes_map[scene_id].start.line > coord.line
                {
                    //scene starts AFTER this line...
                    continue;
                }
                if *scene_id == current_scene_id {
                    break 'scenes;
                }
                // Scene is the first scene that starts BEFORE this line...
                scenes_with_char_speaking.push(scene_id);
                current_scene_id = scene_id;

                let current_forward_idx = scene_ids_in_order.len() - 1 - rev_idx;
                if let Some(next_scene) = scene_ids_in_order.get(current_forward_idx + 1) {
                    current_scene_coordinate = &scenes_map[scene_id].start;
                    break 'scenes;
                }
                // if there isn't a next scene, we are done
                break 'lines;
            }
        }
        if scenes_with_char_speaking.is_empty() {
            return None;
        }
        //panic!();

        Some(scenes_with_char_speaking)
    }

    pub fn get_scenes_on_page_by_nominal_number(
        &self,
        number: &PageNumber,
    ) -> Option<Vec<&SceneID>> {
        unimplemented!();
        None
    }
    pub fn get_scenes_on_page_by_page_id(&self, id: &PageID) -> Option<Vec<&SceneID>> {
        unimplemented!();
        None
    }
    pub fn get_scenes_with_location(&self, location: &LocationID) -> Option<Vec<&SceneID>> {
        let scenes: Vec<&SceneID> = self
            .scenes
            .iter()
            .filter(|(_, scn)| scn.story_locations.contains(&location))
            .map(|(id, _)| id)
            .collect();

        if scenes.is_empty() {
            let location_opt = self.get_location(&location);
            println!("LOCATION: {:?}", location_opt);
            panic!("COULDN'T FIND SCENES FOR LOCATION!");
            return None;
        }
        Some(scenes)
    }

    // ------------ Get PAGEs...
    pub fn get_pages_for_scene(&self, scene_id: &SceneID) -> Option<Vec<usize>> {
        let checked_scene = self.get_scene_from_id(scene_id)?;

        let scenes_ordered = self.get_all_scenes_sorted()?;
        //pages.push(checked_scene.start.page.clone());

        for this_id in scenes_ordered {
            let Some(scene_obj) = self.get_scene_from_id(this_id) else {
                continue; // ERROR -- Tried to find a scene by ID that DOESN'T EXIST in the current script
            };
            if scene_obj.start.page > checked_scene.start.page
                || (scene_obj.start.page == checked_scene.start.page
                    && scene_obj.start.line > checked_scene.start.line)
            {
                let pages: Vec<usize> =
                    (checked_scene.start.page.clone()..=scene_obj.start.page.clone()).collect();
                return Some(pages);
            }
        }

        None
    }

    pub fn get_pages_for_character(&self, character_id: &CharacterID) -> Option<Vec<&PageNumber>> {
        unimplemented!();
        //let Some(checked_character) = self.characters.get(character_id) else {
        //    return None;
        //};
        let mut pages: Vec<&PageNumber> = Vec::new();
        for page in &self.pages {
            let character_lines: Vec<&Line> = page
                .lines
                .iter()
                .filter(|line| match line.line_type {
                    Some(SPType::SP_CHARACTER) | Some(SPType::SP_DUAL_CHARACTERS) => true,
                    _ => false,
                })
                .collect();
            if character_lines.is_empty() {
                continue;
            }
            for line in character_lines {
                let matched_elements: Vec<&TextElement> = line
                    .text_elements
                    .iter()
                    .filter(|te| match te.element_type {
                        Some(SPType::SP_CHARACTER)
                        | Some(SPType::SP_DD_L_CHARACTER)
                        | Some(SPType::SP_DD_R_CHARACTER) => true,
                        _ => false,
                    })
                    .collect();
                let mut current_character_name: String = String::new();
                let mut last_type: Option<SPType> = None;
                let mut matches_character_name = false;
                for element in matched_elements {
                    if element.element_type != last_type {
                        last_type = element.element_type.clone();
                        //if current_character_name == checked_character.name {
                        //    matches_character_name = true;
                        //    break;
                        //}
                        current_character_name = String::new();
                    }
                    if !current_character_name.is_empty() {
                        current_character_name.push(' ');
                    }
                    current_character_name.push_str(&element.text.clone());
                }
                if matches_character_name {
                    if let Some(pagenumber) = &page.page_number {
                        pages.push(pagenumber);
                    } else {
                        // TODO: Add missing page numbers and/or PageIDs during parsing.
                        // Title Page is "0", first content page is "1"
                        //
                        panic!("Each page should be assigned a page number and a page ID.")
                    }
                }
            }
        }

        None
    }
}
