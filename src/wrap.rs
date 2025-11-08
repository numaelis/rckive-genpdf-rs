//! Utilities for text wrapping.

use std::mem;

use crate::style;
use crate::Context;
use crate::Mm;

/// Combines a sequence of styled words into lines with a maximum width.
///
/// If a word does not fit into a line, the wrapper tries to split it using the `split` function.
pub struct Wrapper<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> {
    iter: I,
    context: &'c Context,
    width: Mm,
    x: Mm,
    buf: Vec<style::StyledCow<'s>>,
    has_overflowed: bool,
}

impl<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> Wrapper<'c, 's, I> {
    /// Creates a new wrapper for the given word sequence and with the given maximum width.
    pub fn new(iter: I, context: &'c Context, width: Mm) -> Wrapper<'c, 's, I> {
        Wrapper {
            iter,
            context,
            width,
            x: Mm(0.0),
            buf: Vec::new(),
            has_overflowed: false,
        }
    }

    /// Returns true if this wrapper has overflowed, i. e. if it encountered a word that it could
    /// not split so that it would fit into a line.
    pub fn has_overflowed(&self) -> bool {
        self.has_overflowed
    }
}

impl<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> Iterator for Wrapper<'c, 's, I> {
    // This iterator yields pairs of lines and the length difference between the input words and
    // the line.
    type Item = (Vec<style::StyledCow<'s>>, usize);

    fn next(&mut self) -> Option<(Vec<style::StyledCow<'s>>, usize)> {
        // Append words to self.buf until the maximum line length is reached
        for s in self.iter.by_ref() {
            let mut width = s.width(&self.context.font_cache);

            if self.x + width > self.width {
                // The word does not fit into the current line (at least not completely)

                let mut delta = 0;
                // Try to split the word so that the first part fits into the current line
                let s1 = if let Some((start, end)) = split(self.context, s, self.width - self.x) {
                    // Calculate the number of bytes that we added to the string when splitting it
                    // (for the hyphen, if required).
                    //Choose either Hyphen or Force; you can't choose both. avoid mixing both
                    width = end.width(&self.context.font_cache);
                    if width > self.width {
                        s.into()
                    }else{
                        delta = start.s.len() + end.s.len() - s.s.len();
                        self.buf.push(start);
                        end                        
                    }                                       
                } else {
                    s.into()
                };

                if width > self.width {
                    let mut delta = 0;                   
                    let s2 = if let Some((start, end)) = force_break(self.context, s , self.width - self.x) {                        
                        delta = start.s.len() + end.s.len() - s.s.len();
                        self.buf.push(start);
                        width = end.width(&self.context.font_cache);
                        end
                    } else {
                        s.into()
                    };                        
                    if width > self.width {                        
                        // The remainder of the word is longer than the current page – we will never be
                        // able to render it completely.
                        // TODO: handle gracefully, emit warning                        
                        self.has_overflowed = true;
                        
                        return None;                        
                    }   
                    // Return the current line and add the word that did not fit to the next line
                    let v = std::mem::take(&mut self.buf);
                    self.buf.push(s2);
                    self.x = width;
                    return Some((v, delta));                    
                }

                // Return the current line and add the word that did not fit to the next line
                let v = std::mem::take(&mut self.buf);
                self.buf.push(s1);
                self.x = width;
                return Some((v, delta));
            } else {
                // The word fits in the current line, so just append it
                self.buf.push(s.into());
                self.x += width;
            }
        }

        if self.buf.is_empty() {
            None
        } else {
            Some((mem::take(&mut self.buf), 0))
        }
    }
}

#[cfg(not(feature = "hyphenation"))]
fn split<'s>(
    _context: &Context,
    _s: style::StyledStr<'s>,
    _len: Mm,
) -> Option<(style::StyledCow<'s>, style::StyledCow<'s>)> {
    None
}

/// Tries to split the given string into two parts so that the first part is shorter than the given
/// width.
#[cfg(feature = "hyphenation")]
fn split<'s>(
    context: &Context,
    s: style::StyledStr<'s>,
    width: Mm,
) -> Option<(style::StyledCow<'s>, style::StyledCow<'s>)> {
    use hyphenation::{Hyphenator, Iter};

    let hyphenator = if let Some(hyphenator) = &context.hyphenator {
        hyphenator
    } else {
        return None;
    };

    let mark = "-";
    let mark_width = s.style.str_width(&context.font_cache, mark);

    let hyphenated = hyphenator.hyphenate(s.s);
    let segments: Vec<_> = hyphenated.iter().segments().collect();

    // Find the hyphenation with the longest first part so that the first part (and the hyphen) are
    // shorter than or equals to the required width.
    let idx = segments
        .iter()
        .scan(Mm(0.0), |acc, t| {
            *acc += s.style.str_width(&context.font_cache, t);
            Some(*acc)
        })
        .position(|w| w + mark_width > width)
        .unwrap_or_default();
    if idx > 0 {
        let idx = hyphenated.breaks[idx - 1];
        let start = s.s[..idx].to_owned() + mark;
        let end = &s.s[idx..];
        Some((
            style::StyledCow::new(start, s.style),
            style::StyledCow::new(end, s.style),
        ))
    } else {
        None
    }
}

fn get_idx_width<'s>(
    context: &Context,
    s: style::StyledStr<'s>,
    width: Mm,
    elide_width: Mm,
    segments: &Vec<String>) -> usize{
    let idx: usize = segments
        .iter()
        .scan(Mm(0.0), |acc, t| {
            *acc += s.style.str_width(&context.font_cache, t);
            Some(*acc)
        })
        .position(|w| w + elide_width > width)
        .unwrap_or_default();
    idx
}

//force the word to be split in two and apply elide if necessary or fit size minimum
fn force_break<'s>(
    context: &Context,
    s: style::StyledStr<'s>,
    width: Mm,
) -> Option<(style::StyledCow<'s>, style::StyledCow<'s>)> {    

    let mark= "";
    let mark_width = s.style.str_width(&context.font_cache, mark);
    let elide = "...";
    let elide_width = s.style.str_width(&context.font_cache, elide);
    
    let mut end_elide = style::StyledCow::new("".to_owned(), style::Effect::Bold);
    
    let segments: Vec<_> = s.s
    .chars()
    .map(|c| c.to_string())
    .collect();
    let idx: usize = segments
        .iter()
        .scan(Mm(0.0), |acc, t| {
            *acc += s.style.str_width(&context.font_cache, t);
            Some(*acc)
        })
        .position(|w| w + mark_width > width)
        .unwrap_or_default();
    let indices = s.s.char_indices().map(|(i, _)| i).collect::<Vec<usize>>();
    if idx > 0 {
        let size_fit = s.style.fit_font_size_to();
        let size_font = s.style.font_size();
        if size_fit > 0 && size_fit < size_font{
            //Adjust the size to a minimum value if it is exceeded in the layout.
            let mut size_down = size_font - 1;
            let mut oidx = 0;
            let mut style_down = s.style.clone();
            let mut pass = false;
            while size_down >= size_fit {
                style_down.set_font_size(size_down);
                oidx = get_idx_width(context, style::StyledStr::new(s.s, style_down), width, mark_width, &segments);           
                if oidx == 0 {
                    pass = true;
                    break;
                }
                size_down-=1;
            }
            if pass {
                return Some((
                    style::StyledCow::new(s.s.to_owned(), style_down),
                    style::StyledCow::new("".to_owned(), style_down),
                ));
            }else{    
                //If fit failed, use force cut
                let newidx = get_idx_width(context, style::StyledStr::new(s.s, style_down), width, mark_width, &segments);
                let start = s.s[indices[0]..indices[newidx]].to_owned();
                let end = s.s[indices[newidx]..].to_owned(); 
                
                end_elide = style::StyledCow::new(end, style_down);
                let new_segments = &s.s[indices[newidx]..];
                let new_segments: Vec<_> = new_segments
                                        .chars()
                                        .map(|c| c.to_string())
                                        .collect();    
                let oidx = get_idx_width(context, style::StyledStr::new(s.s, style_down), width, elide_width, &new_segments);
                
                if oidx > 0 {                       
                    let end = s.s[indices[newidx]..indices[newidx + oidx]].to_owned() + elide;
                    end_elide = style::StyledCow::new(end, style_down);           
                }
                Some((
                    style::StyledCow::new(start, style_down),
                    end_elide,
                ))
            }

        }else{
            //use force cut
            let start = s.s[indices[0]..indices[idx]].to_owned();
            let end = s.s[indices[idx]..].to_owned(); 
            
            end_elide = style::StyledCow::new(end, s.style);
            let new_segments = &s.s[indices[idx]..];
            let new_segments: Vec<_> = new_segments
                                    .chars()
                                    .map(|c| c.to_string())
                                    .collect();    
            let oidx = get_idx_width(context, s, width, elide_width, &new_segments);
            
            if oidx > 0 {                       
                let end = s.s[indices[idx]..indices[idx + oidx]].to_owned() + elide;
                end_elide = style::StyledCow::new(end, s.style);           
            }
            Some((
                style::StyledCow::new(start, s.style),
                end_elide,
            ))
        }
    } else {
        None
    }
}

/// Splits a sequence of styled strings into words.
pub struct Words<I: Iterator<Item = style::StyledString>> {
    iter: I,
    s: Option<style::StyledString>,
}

impl<I: Iterator<Item = style::StyledString>> Words<I> {
    /// Creates a new words iterator.
    pub fn new<IntoIter: IntoIterator<Item = style::StyledString, IntoIter = I>>(
        iter: IntoIter,
    ) -> Words<I> {
        Words {
            iter: iter.into_iter(),
            s: None,
        }
    }
}

impl<I: Iterator<Item = style::StyledString>> Iterator for Words<I> {
    type Item = style::StyledString;

    fn next(&mut self) -> Option<style::StyledString> {
        if self.s.as_ref().map(|s| s.s.is_empty()).unwrap_or(true) {
            self.s = self.iter.next();
        }

        if let Some(s) = &mut self.s {
            // Split at the first space or use the complete string
            let n = s.s.find(' ').map(|i| i + 1).unwrap_or_else(|| s.s.len());
            let mut tmp = s.s.split_off(n);
            mem::swap(&mut tmp, &mut s.s);
            Some(style::StyledString::new(tmp, s.style))
        } else {
            None
        }
    }
}
