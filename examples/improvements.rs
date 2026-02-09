//! This example generates a demo PDF document and writes it to the path that was passed as the
//! first command-line argument.  You may have to adapt the `FONT_DIRS`, `DEFAULT_FONT_NAME` and
//! `MONO_FONT_NAME` constants for your system so that these files exist:
//! - `{FONT_DIR}/{name}-Regular.ttf`
//! - `{FONT_DIR}/{name}-Bold.ttf`
//! - `{FONT_DIR}/{name}-Italic.ttf`
//! - `{FONT_DIR}/{name}-BoldItalic.ttf`
//! for `name` in {`DEFAULT_FONT_NAME`, `MONO_FONT_NAME`}.

//improvements 2025 - 2026


use std::env;

use rckive_genpdf::Alignment;
use rckive_genpdf::Element as _;
use rckive_genpdf::{elements, fonts, style};
use rckive_genpdf::Margins;
use rckive_genpdf::Position;
use rckive_genpdf::style::BackgroundStyle;


const FONT_DIRS: &[&str] = &[
    "/usr/share/fonts/liberation",
    "/usr/share/fonts/truetype/liberation",
];
const DEFAULT_FONT_NAME: &str = "LiberationSans";
const MONO_FONT_NAME: &str = "LiberationMono";

fn main() {
    let font_dir = FONT_DIRS
        .iter()
        .find(|path| std::path::Path::new(path).exists())
        .expect("Could not find font directory");
    let default_font =
        fonts::from_files(font_dir, DEFAULT_FONT_NAME, Some(fonts::Builtin::Helvetica))
            .expect("Failed to load the default font family");
    let monospace_font = fonts::from_files(font_dir, MONO_FONT_NAME, Some(fonts::Builtin::Courier))
        .expect("Failed to load the monospace font family");
    
    ////////// Frame Element dash //////
    let mut doc = rckive_genpdf::Document::new(default_font.clone());
    doc.set_title("rckive_genpdf Demo Document");
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);

    let monospace = doc.add_font_family(monospace_font);
    let code = style::Style::from(monospace).bold();
    let red = style::Color::Rgb(255, 0, 0);
    let blue = style::Color::Rgb(0, 0, 255);

        
    let line_style = style::LineStyle::new().with_thickness(0.2)
                                                .with_color(style::Color::Greyscale(125))
                                                .with_dash(3).with_dash2(1)
                                                .with_gap(1).with_gap2(0);                                                   
        
    doc.push(
        elements::Paragraph::new("rckive_genpdf Demo Document")
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(20))
            .framed(line_style.clone()),
    );            
    
    doc.push(elements::Break::new(1.5));
    
    
    let frame_top_bottom = elements::FramedElement::with_line_style_trbl(
            elements::Paragraph::new("frame with_line_style_trbl"),
            line_style.clone(), true, false, true, false
    );
    
    doc.push(frame_top_bottom);
    
    doc.render_to_file("frame_dash_sides.pdf")
        .expect("Failed to write output file");
        
    /////////////   Page Frame /////////////
        
    let mut doc = rckive_genpdf::Document::new(default_font.clone());
    doc.set_title("rckive_genpdf Demo Document");
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);
    
    let mut decorator = rckive_genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    decorator.set_header(|page| {
        let mut layout = elements::LinearLayout::vertical();
        if page > 1 {
            layout.push(
                elements::Paragraph::new(format!("Page {}", page)).aligned(Alignment::Center),
            );
            layout.push(elements::Break::new(1.));
        }
        layout.styled(style::Style::new().with_font_size(10))
    });
    doc.set_page_decorator(decorator);
    
    let line_style = style::LineStyle::new().with_thickness(0.2).with_color(style::Color::Greyscale(125));
    
    doc.set_page_frame_line_style(line_style.clone());
    doc.set_header_frame_line_style(line_style.clone());
    
    doc.push(elements::PageBreak::new());
    
    doc.render_to_file("page_frame.pdf")
        .expect("Failed to write output file");
        
        
    /////////// footer ///////
        
    let mut doc = rckive_genpdf::Document::new(default_font.clone());
    doc.set_title("rckive_genpdf Demo Document");
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);
    
    let mut decorator = rckive_genpdf::SimplePageDecorator::new();
    
    //Currently, the footer is created manually using layout orphan.
        
    let x = 0;
    //A4 height = 297.0;
    //A4 widht = 210.0;
    let mut y = 297.0 - 10.0;
    let width_area =  210.0 - 10.0 - 10.0;  
    
    let mut paragraph_footer = elements::Paragraph::new("footer paragraph")
                            .aligned(Alignment::Center);
                            
    let hp = paragraph_footer.get_height(doc.context(), width_area);
    let hp: f32 = hp.into();
    y -= hp;
    decorator.set_margins(Margins::trbl(10,10,hp+10.0,10));
    decorator.set_footer(move |page| {
         let mut layout = elements::LinearLayout::vertical().with_orphan(true);
            
        layout.set_orphan_position(x, y);
        let paragraph_footer = paragraph_footer.clone();
        layout.push(paragraph_footer);
        layout
    });
    doc.set_page_decorator(decorator);
    
    let line_style = style::LineStyle::new().with_thickness(0.2).with_color(style::Color::Greyscale(125));
    
    doc.set_page_frame_line_style(line_style.clone());
    doc.set_header_frame_line_style(line_style.clone());
    
    // footer frame Manual
    doc.set_rec_footer(Position::new(x,y), Position::new(width_area, hp));
    doc.set_footer_frame_line_style(line_style.clone());
    
    doc.push(elements::PageBreak::new());
    
    doc.render_to_file("footer.pdf")
        .expect("Failed to write output file");
        
    
    /////////// fit_font_size //////////
    
    let mut doc = rckive_genpdf::Document::new(default_font.clone());
    doc.set_title("rckive_genpdf Demo Document");
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);
    let mut decorator = rckive_genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);
    let mut table = elements::TableLayout::new(vec![1, 5]);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));
    table
        .row()
        .element(
            elements::Paragraph::new("textlargelargelargelargelargelargelarge")
                .styled(style::Style::new().with_font_size(40).with_fit_font_size_to(5))
                .padded(1),
        )
        .element(
            elements::Paragraph::new("Text")
                .styled(style::Effect::Bold)
                .padded(1),
        )
        .push()
        .expect("Invalid table row");
    doc.push(table);
    doc.render_to_file("fit_font_size.pdf")
        .expect("Failed to write output file");
        
        
    /////////// background element //////////
    
    let mut doc = rckive_genpdf::Document::new(default_font.clone());
    doc.set_title("rckive_genpdf Demo Document");
    doc.set_minimal_conformance();
    doc.set_line_spacing(0.9);
    let mut decorator = rckive_genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);    
    
    let mut layout = elements::LinearLayout::vertical();
    
    layout.push(
                elements::Paragraph::new("Name")
                    .styled(style::Effect::Bold)  
                    .padded(1)
            );
    
    let line_style = style::LineStyle::new().with_thickness(0.0);
    let mut layout = elements::FramedElement::with_line_style_trbl(
            layout,
            line_style.clone(), false, false, false, false
    );
    
    layout.set_background(true, BackgroundStyle::new().with_color(style::Color::Rgb(200, 200, 255)));
    
    
    let mut layout2 = elements::FramedElement::with_line_style_trbl_and_background(
                elements::LinearLayout::vertical().element( 
                    elements::Paragraph::new("Text")
                            .styled(style::Effect::Bold)  
                            .padded(1)
                ),
                style::LineStyle::new().with_thickness(0.0), false, false, false, false,
                true, BackgroundStyle::new().with_color(style::Color::Rgb(200, 240, 120))
    );
    
    
    let mut table = elements::TableLayout::new(vec![1, 5]);
    table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));
    table
        .row()
        .element(
            layout
            )
        .element(
            layout2
        )
        .push()
        .expect("Invalid table row");
    doc.push(table);
    doc.render_to_file("background.pdf")
        .expect("Failed to write output file");
}
