# Wonder Land 

This is a mini browser project. I started working on this initially for Andreas Kling's Browser Jam held from Sept 13 - Sept 15 2024. It didn't make the submission but I intend to keep working on it atleast as a way to learn about how browsers work and also learn about rust.

It's called Wonder Land because I don't really know what I'm doing. This is a learning process.

## Parts
### Alice -- The HTML parser
The first component is the html parser. This parses the html string into DOM objects. It consists of two parts;
- Tokenizer -- reads the html string into tokens that the parser can use
- Parser -- does the actual parsing
I'm working with the [official html spec](https://html.spec.whatwg.org/) as my guide, so this parser *should* be spec compliant


## Progress
As of last commit, I have a html tokenizer that can tokenize
```html
<!DOCTYPE html>
<html>

</html>
```
