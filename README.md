# Screenplay Doc Parser
Parses a PDF document file into a structured, semantically typed ScreenplayDocument object.

This parser currently supports parsing from PDF, but may include support for other formats such as FDX or Fountain in the future.

## How

The PDF parser uses the x,y positions of the `TextElements` on a page to deduce their type. This will usually be correct, BUT may require manual intervention after parsing for some edge-cases. Screenwriters love to play with formatting and indentation...

In general, screenplay elements like Action, Character, Dialogue, Parentheticals, even the Page Number, Scene Numbers and revision markers, all have a set indentation point, and/or specific justification.

Also, screenplays generally have consistent margins, or at least margins consistint within the same document (hopefully...)

If we know the indentations and margins of a document, we can deduce that, any line of text which begins at 1.5 inches from the left side, is below the top margin and above the bottom margin, is probably an `Action` line.

Lines that adhere to the above, but also start with something like `INT.` or `EXT.` are very likely `SceneHeading`s.

Character names and dialogue have their own indentations, as well as parentheticals. So this scheme should yield correct parsing for the majority of a properly-formatted ScreenplayPDF.

The user of this crate can also pass in their own indentation values and strings to match against for Scene Environments or Time of Day (INT./EXT., DAY, NIGHT...), so we can even support screenplays that are A4, or have deviated somewhat from "standard" US-Letter formatting.

The default margins and indentations for this crate are taken from the default settings found in Final Draft 11, for a simple US-Letter screenplay.

## What

This categorizes the following Screenplay Element Types:

- Action
- Character Cue
    - Also Character Extensions (i.e. the `(V.O.)` in `CHARACTER (V.O.)`)
- Dialogue
- Parenthetical
- Scene Headings (including Heading Elements)
    - Scene Environment (INT. or EXT.)
    - Scene Location
        - Scene Sublocation (any element that follows a Location which ISN'T another valid element...)
    - Time of Day (DAY, NIGHT, MORNING, EVENING...)

This parser also captures the following screenplay elements as metadata
- Scene Number (alphanumeric)
- Page Number (alphanumeric)

### Types that rely on matching arbitrary strings

Some types, such as `TimeOfDay`, `Revision Markers`, and `Environment` rely on arbitrary string values. You can pass in your own collection of these strings, to parse a screenplay written in a different language, or support additional / specific elements.

For example, you can add "DUSK" or "HIGH NOON" as `TimeOfDay` strings, so that they are correctly identified as `TimeOfDay` elements

### Indentations

Additionally, the `ElementIndentations` struct can be passed in to the PDF parser, to provide custom indentations and support parsing a screenplay formatted in A4, or a screenplay formatted with "centered" (as in placement, not justification) sctipts, like from Fade In or other programs.

## TODO

These are currently not parsed or handled properly yet:

- Title Page elements
- Dual Dialogue blocks
- Transitions ("CUT TO:", "FADE IN:", etc; any element like that which is right-aligned.)
- A4 detection (no default ElementIndentation values for A4 yet)

# DEPENDENCIES

This parser has an optional feature, which uses the `mupdf-basic-text-extractor` crate to allow PDF file reading. You may choose to exclude this feature and roll your own PDF file-parsing, and then handle the conversion to the generic `pdf_document::PDFDocument' object, which gets passed into the PDF parser. 

# LICENSE

This code is licensed under AGPL-3.0.