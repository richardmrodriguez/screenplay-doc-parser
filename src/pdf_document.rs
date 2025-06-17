// TODO: impl defaults for standard US-LETTER indentations
#[derive(Default)]
pub struct ElementIndentationsInches {
    pub pagewidth: f64,
    pub pageheight: f64,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,

    pub action: f64,
    pub character: f64,
    pub dialogue: f64,
    pub parenthetical: f64,
}

#[derive(Default)]
pub struct ElementIndentationsPoints {
    pub pagewidth: f64,
    pub pageheight: f64,
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,

    pub action: f64,
    pub character: f64,
    pub dialogue: f64,
    pub parenthetical: f64,
}

pub fn get_us_letter_default_indentations_inches() -> ElementIndentationsInches {
    ElementIndentationsInches {
        top: 10.0,
        bottom: 1.0,
        left: 1.5,
        right: 7.5,
        pageheight: 11.0,
        pagewidth: 8.5,
        action: 1.5,
        character: 3.7,
        dialogue:2.5,
        parenthetical: 3.1,
    }
}

pub fn get_indentations_inches_as_pts(indentations: &ElementIndentationsInches, resolution: &Option<f64>) -> ElementIndentationsPoints {
    let mut current_resolution: f64 = 72.0;
    if let Some(r) = resolution {
        current_resolution = r.clone()
    }
    
    ElementIndentationsPoints {
        top: indentations.top * current_resolution,
        bottom: indentations.bottom * current_resolution,
        pagewidth: indentations.pagewidth * current_resolution,
        pageheight: indentations.pageheight * current_resolution,
        left: indentations.left * current_resolution,
        right: indentations.right *current_resolution,
        action: indentations.action * current_resolution,
        character: indentations.character * current_resolution,
        dialogue: indentations.dialogue *current_resolution,
        parenthetical: indentations.parenthetical * current_resolution
    }
}

pub fn get_us_letter_default_indentation_pts() -> ElementIndentationsPoints {
    get_indentations_inches_as_pts(&get_us_letter_default_indentations_inches(), &None)
}

#[derive(Default, Clone, Copy)]
pub struct TextPosition {
    pub x: f64,
    pub y: f64,
}
#[derive(Default)]
pub struct PageSize {
    pub width: f64,
    pub height: f64,
}
#[derive(Default)]
pub struct Word {
    pub text: String,
    pub text_bbox_width: f64,
    pub position:TextPosition,
    pub font_name: Option<String>,
    pub font_size: f64,
    pub font_character_width: f64,
}
#[derive(Default)]
pub struct Line {
    pub words:Vec<Word>
}
#[derive(Default)]
pub struct Page {
    pub lines: Vec<Line>,
    pub page_size: PageSize
}
#[derive(Default)]
pub struct PDFDocument {
    pub pages: Vec<Page>,
    pub pdf_creator: Option<String>,
}
