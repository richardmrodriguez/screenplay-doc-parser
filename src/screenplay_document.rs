use std::{collections::HashMap, default, hash::Hash, ops::{Deref, DerefMut}, time::SystemTime};
use serde::de::IntoDeserializer;
use uuid::{Uuid};
use crate::{pdf_document, screenplay_document};


#[derive(PartialEq, Clone, Debug)]
pub enum TimeOfDay {
    Day(String),
    Night(String),
    Morning(String),
    Evening(String),
    Afternoon(String),
    Extras(Option<HashMap<String, String>>)
}
impl TimeOfDay {
    fn get(&self) {

    }
}


#[derive(PartialEq, Clone, Debug)]
pub struct TimeOfDayCollection {
    pub day: TimeOfDay,
    pub night: TimeOfDay,
    pub morning: TimeOfDay,
    pub evening: TimeOfDay,
    pub afternoon: TimeOfDay,
    pub extras: Option<HashMap<String, String>>

}
impl Default for TimeOfDayCollection {
    fn default() -> Self {
        return Self { 
            day: TimeOfDay::Day("DAY".into()), 
            night: TimeOfDay::Night("NIGHT".into()), 
            morning: TimeOfDay::Morning("MORNING".into()), 
            evening: TimeOfDay::Evening("EVENING".into()), 
            afternoon: TimeOfDay::Afternoon("AFTERNOON".into()), 
            extras: None };
    }
}
impl TimeOfDayCollection {
    
    pub fn is_time_of_day(&self, target: &String) -> bool {
        let vars: Vec<&TimeOfDay> = vec![
            &self.day,
            &self.night,
            &self.morning,
            &self.evening,
            &self.afternoon
        ];

        for time in vars {
            match time {
                TimeOfDay::Day(string)
                |TimeOfDay::Night(string)
                |TimeOfDay::Morning(string)
                |TimeOfDay::Evening(string)
                |TimeOfDay::Afternoon(string) => {
                    if string == target{
                        return true;
                    }
                }
                _ => {}   
            }
            
        }

        match &self.extras {
            None => {return false;},
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

    pub fn get_time_of_day(&self, target: &String) -> Option<TimeOfDay>{
        let vars: Vec<&TimeOfDay> = vec![
            &self.day,
            &self.night,
            &self.morning,
            &self.evening,
            &self.afternoon
        ];

        for time in vars {
            match time {
                TimeOfDay::Day(string)
                |TimeOfDay::Night(string)
                |TimeOfDay::Morning(string)
                |TimeOfDay::Evening(string)
                |TimeOfDay::Afternoon(string) => {
                    if string == target{
                        return Some(time.clone());
                    }
                }
                _ => {}   
            }
            
        }

        match &self.extras {
            None => {return None;},
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
#[derive(Default, Clone, Debug,
    Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum SPType {
    SP_ACTION = 0,

    SP_CHARACTER,
    SP_CHARACTER_EXTENSION, // require context to parse (previous word type)
    SP_DG_MORE_CONTINUED, // specifically has MORE or CONTINUED or CONT'D within parentheses
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

    SP_PAGENUM, // Nominal page number
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
    _TYPECOUNT
}



// -------- SCREENPLAY TYPED STRUCTS / ENUMS

// -------------------- CHARACTER
#[derive(Default, PartialEq, Clone, Debug, Eq, Hash)]
pub struct CharacterID(Uuid);

#[derive(Default, PartialEq, Clone, Debug, Eq, Hash)]
pub struct Character {
    name: String,
    id: CharacterID
}

// -------------------- PAGE
#[derive(Default, PartialEq, Clone, Debug, Eq, Hash)]
pub struct PageNumID(pub Uuid);
impl Deref for PageNumID {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }

}

impl DerefMut for PageNumID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
} 


#[derive(Default, PartialEq, Clone, Debug)]
pub struct PageNumber(pub String);

// -------------------- SCENE 
#[derive(Default, PartialEq, Clone, Debug)]
pub struct SceneNumber(pub String);

#[derive(Default, PartialEq, Clone, Debug, Hash, Eq)]
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
    Separator, // hyphen
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

    pub story_location: Location,
    pub story_sublocation: Option<Location>,
    pub story_time_of_day: Option<TimeOfDay>, // DAY, NIGHT, etc.
}

pub struct EnvironmentStrings {
    pub int: Vec<String>,
    pub ext: Vec<String>,
    pub combo: Vec<String>
}
impl Default for EnvironmentStrings {
    fn default() -> Self {
        EnvironmentStrings {
            int: vec![
                "INT.".into()
            ],
            ext: vec![
                "EXT.".into()
            ],
            combo: vec![
                "INT./EXT.".into(),
                "I./E.".into(),
                "EXT./INT.".into(),
                "E./I.".into()
            ]
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
        if current_env_strs.ext.contains(&string)  {
            return Some(Environment::Ext);
        }
        if current_env_strs.combo.contains(&string) { // TODO: actually hanndle combos (4 total possibilities, int/int, int/ext, ext/int, and ext/ext)
            return Some(Environment::Combo(None))
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

#[derive(Default, PartialEq, Clone, Debug, Eq, Hash)]
pub struct Location {
    pub strings: Vec<String>,
    pub sublocations: Option<Vec<Uuid>>, // list of IDs for other locations
    pub superlocation: Option<Uuid> // 

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

#[derive(Default, PartialEq, Clone, Debug)]
    pub struct ScreenplayCoordinate {
        pub page: u64,
        pub line: u64,
        pub element: Option<u64>
    }



#[derive(Default, PartialEq, Clone, Debug)]
pub struct ScreenplayDocument {
    pub pages: Vec<Page>,
    pub revisions: Option<Vec<String>>, // current (and possible previous) revision date(s) from the title page
    pub scenes: HashMap<SceneID, Scene>,
    pub locations: HashMap<LocationID, Location>,
    pub characters: HashMap<CharacterID, Character>
}
impl ScreenplayDocument {
    // ------------ Get LINE...
    pub fn get_lines_for_character_speaking(&self, character: &CharacterID) -> Option<Vec<&ScreenplayCoordinate>> {
        None
    }

    // ------------ Get CHARACTERS...
    pub fn get_characters_for_scene(&self, scene_id: &SceneID) -> Option<Vec<&CharacterID>> {
        None
    }
    
    pub fn get_characters_for_page(&self, page_num_id: &PageNumID) -> Option<Vec<&Character>> {
        None
    }

    pub fn get_characters_for_scene_heading_element(&self, scene_heading_element: &SceneHeadingElement) -> Option<Vec<&Character>> {
        None
    }
    
    // ------------ Get SCENES...
    pub fn get_scenes_from_ids(&self, ids: &Vec<&SceneID>) -> Option<Vec<&SceneID>> {
        None
    }

    pub fn get_scenes_with_element(&self, element: &TextElement) -> Option<Vec<&SceneID>> {
        None
    }

    pub fn get_scenes_with_character_speaking(&self, character: &Character,) -> Option<Vec<&SceneID>>{
        None
    }

    pub fn get_scenes_with_scene_heading_element(&self, s: &screenplay_document::SceneHeadingElement) -> Option<Vec<&SceneID>> {
        None
    }
    pub fn get_scenes_on_page_by_nominal_number(&self, number: &PageNumber) -> Option<Vec<&SceneID>> {
        None
    }
    pub fn get_scenes_on_page_by_id(&self, id: &PageNumID) -> Option<Vec<&SceneID>> {
        None
    }
    pub fn get_scene_for_screenplay_coordinate(&self, screenplay_coordinate: &ScreenplayCoordinate) -> Option<&SceneID> {
        None
    }

    // ------------ Get PAGEs...
    pub fn get_pages_for_scene(&self, scene_id: &SceneID) -> Option<Vec<&PageNumID>> {
        None
    }

    pub fn get_pages_for_character(&self, character_id: &CharacterID) -> Option<Vec<&PageNumID>> {
        None
    }

    pub fn get_pages_for_scene_heading_element(&self, scene_heading_element: &SceneHeadingElement) -> Option<Vec<&PageNumID>> {
        None
    }

    pub fn get_page_from_screenplay_coordinate(&self, screenplay_coordinate: &ScreenplayCoordinate) -> Option<&PageNumID> {
        None
    }

    
}
