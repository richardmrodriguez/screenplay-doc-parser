use std::{collections::HashMap, default, hash::Hash, time::SystemTime};
use uuid::{Uuid};
use crate::pdf_document;

#[derive(PartialEq, Clone, Debug)]
pub struct TimeOfDay {
    day: String,
    night: String,
    morning: String,
    evening: String,
    afternoon: String,
    extras: Option<HashMap<String, String>>

}
impl Default for TimeOfDay {
    fn default() -> Self {
        return Self { 
            day: "DAY".into(), 
            night: "NIGHT".into(), 
            morning: "MORNING".into(), 
            evening: "EVENING".into(), 
            afternoon: "AFTERNOON".into(), 
            extras: None };
    }
}
impl TimeOfDay {
    pub fn is_time_of_day(&self, target: &String) -> bool {
        if target == &self.morning
        || target == &self.day 
        || target == &self.afternoon
        || target == &self.evening
        || target == &self.night

        {
            return true;
        }

        match &self.extras {
            None => {return false;},
            Some(e) => {
                for (id, string) in e {
                    if target == string {
                        return true;
                    }
                }
            }
        }

        return false;
            
        
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
#[derive(Default, PartialEq, Clone, Copy, Debug)]
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
    SP_SCENE_HEADING, // begins with INT. , EXT. , or I./E.
    
    /// `INT.`, `EXT.`, `INT./EXT.`, etc.
    SP_ENVIRONMENT, 
    SP_LOCATION,
    SP_SCENE_HEADING_SUB_ELEMENT,
    SP_SCENE_HEADING_SEPARATOR, /// Breaks up a slugline -- EXT. BASEBALL FIELD - PITCHER'S MOUND - PAST - NIGHT
    SP_SCENE_TIMEPERIOD, // PAST, PRESENT, FUTURE, arbitrary timeframe "BEFORE DINNER", "AFTER THE EXPLOSION", etc.
    SP_SUBLOCATION,
    SP_TIME_OF_DAY,

    SP_SHOT_ANGLE, // SHOT or ANGLE on something, NOT a full scene heading / location

    SP_PAGENUM, // Nominal page number
    SP_SCENENUM, // Nominal scene number

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
    pub line_type: Option<SPType>, // should default to NONE when initialized!!!
    pub preceding_empty_lines: u64,
    pub revised: bool,
    pub blank: bool,
}


#[derive(Default, PartialEq, Clone, Debug)]
pub struct Page {
    pub lines: Vec<Line>,
    pub page_number: Option<String>,
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
pub struct Scene {
    pub start: ScreenplayCoordinate,

    pub number: String,
    pub revised: bool,

    pub story_location: Location,
    pub story_sublocation: Option<Location>,
    pub story_time_of_day: String, // DAY, NIGHT, etc.
}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct Location {
    pub elements: Vec<TextElement>,
    pub sublocations: Vec<Uuid> // list of IDs for other locations

}

#[derive(Default, PartialEq, Clone, Debug)]
pub struct ScreenplayDocument {
    pub pages: Vec<Page>,
    pub revisions: Option<Vec<SystemTime>>, // current (and possible previous) revision date(s) from the title page
    pub scenes: HashMap<Uuid, Scene>,
    pub locations: HashMap<Uuid, Location>
}
