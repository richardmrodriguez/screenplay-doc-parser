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
impl ElementIndentationsInches {
    pub fn us_letter_default() -> Self {
        ElementIndentationsInches {
            top: 10.0,
            bottom: 1.0,
            left: 1.5,
            right: 7.25, // Final Draft default???
            pageheight: 11.0,
            pagewidth: 8.5,
            action: 1.5,
            character: 3.7,
            dialogue:2.5,
            parenthetical: 3.1,
        }
    }

    pub fn top(mut self, new_top: f64) -> Self {
        self.top = new_top;
        self
    }

    pub fn bottom(mut self, new_bottom: f64) -> Self {
        self.bottom = new_bottom;
        self
    }
    pub fn left(mut self, new_left: f64) -> Self {
        self.left = new_left;
        self
    }
    pub fn right(mut self, new_right: f64) -> Self {
        self.right = new_right;
        self
    }

    //

    pub fn pageheight(mut self, new_pageheight: f64) -> Self {
        self.pageheight = new_pageheight;
        self
    }
    pub fn pagewidth(mut self, new_pagewidth: f64) -> Self {
        self.pagewidth = new_pagewidth;
        self
    }

    pub fn action(mut self, new_action: f64) -> Self {
        self.action = new_action;
        self
    }
    pub fn character(mut self, new_character: f64) -> Self {
        self.character = new_character;
        self
    }
    pub fn dialogue(mut self, new_dialogue: f64) -> Self {
        self.dialogue = new_dialogue;
        self
    }
    pub fn parenthetical(mut self, new_parenthetical: f64) -> Self {
        self.parenthetical = new_parenthetical;
        self
    }

    pub fn from_points(indentations: &ElementIndentationsPoints, resolution: &f64) -> ElementIndentationsInches {
        ElementIndentationsInches {
            top: indentations.top / resolution,
            bottom: indentations.bottom / resolution,
            pagewidth: indentations.pagewidth / resolution,
            pageheight: indentations.pageheight / resolution,
            left: indentations.left / resolution,
            right: indentations.right / resolution,
            action: indentations.action / resolution,
            character: indentations.character / resolution,
            dialogue: indentations.dialogue / resolution,
            parenthetical: indentations.parenthetical / resolution
        }
    }
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
impl ElementIndentationsPoints {

    /// Gets a default struct of Indentations in Points for US-Letter formatted screenplays.
    /// 
    /// Takes an optional resolution. `None` will use a default of 72.0 point-per-inch resolution.
    pub fn us_letter_default(resolution: &Option<f64>) -> Self {
        let mut current_resolution: f64 = 72.0;
        if let Some(r) = resolution {
            current_resolution = r.clone()
        }

        let indentations = ElementIndentationsInches::us_letter_default();
        
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

    pub fn from_inches(indentations: &ElementIndentationsInches, resolution: &Option<f64>) -> ElementIndentationsPoints {
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
}





#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct TextPosition {
    pub x: f64,
    pub y: f64,
}
#[derive(Default, Debug)]
pub struct PageSize {
    pub width: f64,
    pub height: f64,
}
#[derive(Default, Debug)]
pub struct Word {
    pub text: String,
    pub bbox_width: f64,
    pub bbox_height: f64,
    pub position:TextPosition,
    pub font_name: Option<String>,
    pub font_size: f64,
    pub font_character_width: f64,
}
#[derive(Default, Debug)]
pub struct Line {
    pub words:Vec<Word>
}
#[derive(Default, Debug)]
pub struct Page {
    pub lines: Vec<Line>,
    pub page_size: PageSize
}
#[derive(Default)]
pub struct PDFDocument {
    pub pages: Vec<Page>,
    pub pdf_creator: Option<String>,
}
