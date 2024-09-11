package main

import (
	"bytes"
	_ "embed"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"strings"
)

type bucket struct {
	name string
	objs []object
}

func newBucket(name string, objs ...object) bucket {
	return bucket{name, objs}
}

type object struct {
	objectKey       string
	objectType      fixtureObjectType
	multipleVersion bool
}

type fixtureObjectType int

type fixtureObjectMap map[fixtureObjectType][]byte

const (
	smallText fixtureObjectType = iota
	mediumText
	largeText
	smallHtml
	rustCode
	imagePng
	imageJpg
	empty
)

func buildObjectMap() fixtureObjectMap {
	return fixtureObjectMap{
		smallText:  smallTextObject(),
		mediumText: mediumTextObject(),
		largeText:  largeTextObject(),
		smallHtml:  smallHtmlObject(),
		rustCode:   rustCodeObject(),
		imagePng:   imagePngObject(),
		imageJpg:   imageJpgObject(),
		empty:      emptyObject(),
	}
}

func smallTextObject() []byte {
	return []byte("test")
}

func mediumTextObject() []byte {
	return []byte(`Lasciatemi morire,
Lasciatemi morire;
E che volete voi che mi conforte
In così dura sorte,
In così gran martire?
Lasciatemi morire.

O Teseo, o Teseo mio,
sì, che mio ti vo dir che mio pur sei
benche t'involi, hai crudo! a gl'occhi miei.
Volgiti, Teseo mio, volgiti, Teseo, o Dio,
volgiti in dietro a rimirar colei
che lasciato per te la patria e'l regno
e'n quest' arene ancora
cibo di fere dispietate e crude,
lasciera l'ossa ignude.
O Teseo, o Teseo mio,
se tu sapessi, oh Dio!
se tu sapessi, ohime!
come s'affanna
la povera Ariana,
forse, forse pentito
rivolgeresti ancor la prora lito.
Ma con l'aure serene
tu te ne vai felice
ed io qui piango:
A te prepara Attene
liete pompe superbe,
ed io rimango
cibo di fere in solitarie arene.
Tu l'un el'altro tuo vecchio parente
Stringerai lieto ed io
più non vedrovi o madre o padre mio.

Dove è la fede
che tanto mi giuravi?
Così ne'l alta sede
tu mi ripon de gl'avi?
Son queste le corone
onde m'adorni il crine?
Questi li scetri sono?
Queste le gemme e gl'ori?
Lasciarmi in abandono
a fere che mi stracci e mi divori?
Ah Teseo, ah Teseo mio,
lascierai tu morire in van piangendo, in van gridando aita
la misera Ariana ch'a te fidossi e ti die' gloria e vita?

Ahi, che non pur risponde!
Ahi, che piu d'aspe è sord' a miei lamenti!
O nembi, o turbi, o venti
sommergetelo voi dentro a quell' onde!
Correte, orchi e balene,
e de le membra immonde
empiete le voragini profonde!
Che parlo, ahi, che vaneggio?
Misera, ohime! che chieggio?
O Teseo, o Teseo mio,
non son quell' io
che i feri detti sciolse, parlò l'affanno mio, parlò il dolore,
parlò la lingua, sì, ma non gia il core.`)
}

func largeTextObject() []byte {
	text := strings.Repeat("0123456789\n", 100000)
	return []byte(text)
}

func smallHtmlObject() []byte {
	html := `<!DOCTYPE html>
<html lang="en">
  <head>
	<meta charset="utf-8">
	<title>title</title>
	<link rel="stylesheet" href="style.css">
	<script src="script.js"></script>
  </head>
  <body>hello</body>
</html>`
	return []byte(html)
}

func rustCodeObject() []byte {
	code := `use chrono::{DateTime, Local};

pub const APP_NAME: &str = "STU";

#[derive(Clone, Debug)]
pub struct BucketItem {
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum ObjectItem {
    Dir {
        name: String,
    },
    File {
        name: String,
        size_byte: usize,
        last_modified: DateTime<Local>,
    },
}

impl ObjectItem {
    pub fn last_modified(&self) -> Option<DateTime<Local>> {
        match self {
            ObjectItem::Dir { .. } => None,
            ObjectItem::File { last_modified, .. } => Some(*last_modified),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;	
	
    #[test]
    fn test_object_item() {
        let obj = ObjectItem::File {
            name: "file.txt".to_string(),
            size_byte: 100,
            last_modified: Local::now(),
        };
        assert_eq!(obj.last_modified().is_some(), true);
    }
}
`
	return []byte(code)
}

func imagePngObject() []byte {
	var buf bytes.Buffer
	png.Encode(&buf, dummyImage())
	return buf.Bytes()
}

//go:embed fixture/icon.jpg
var iconImage []byte

func imageJpgObject() []byte {
	return iconImage
}

func dummyImage() image.Image {
	w, h := 600, 400
	img := image.NewRGBA(image.Rect(0, 0, w, h))
	for x := 0; x < w; x++ {
		for y := 0; y < h; y++ {
			var c color.Color
			if x < w/2 && y < h/2 {
				c = color.RGBA{R: 128, G: 128, B: 0, A: 100}
			} else if x >= w/2 && y < h/2 {
				c = color.RGBA{R: 128, G: 0, B: 128, A: 100}
			} else if x < w/2 && h >= y/2 {
				c = color.RGBA{R: 0, G: 128, B: 128, A: 100}
			} else {
				c = color.RGBA{R: 128, G: 128, B: 128, A: 100}
			}
			img.Set(x, y, c)
		}
	}
	return img
}

func emptyObject() []byte {
	return []byte{}
}

func defaultFixture() []bucket {
	return []bucket{
		newBucket("test-bucket-1", hierarchicalObjects()...),
		newBucket("test-bucket-2", variousContentTypeObjects()...),
		newBucket("test-bucket-3", largeNumberObjects()...),
		newBucket("test-bucket-4"),
		newBucket("test-bucket-5"),
		newBucket("test-bucket-6"),
		newBucket("test-bucket-7"),
		newBucket("test-bucket-8"),
		newBucket("test-bucket-9"),
		newBucket("test-bucket-a"),
		newBucket("test-bucket-b"),
		newBucket("test-bucket-c"),
		newBucket("test-bucket-d"),
		newBucket("test-bucket-e"),
		newBucket("test-bucket-f"),
		newBucket("test-bucket-g"),
		newBucket("test-bucket-h"),
		newBucket("test-bucket-i"),
		newBucket("test-bucket-j"),
		newBucket("test-bucket-k"),
		newBucket("test-bucket-l"),
		newBucket("test-bucket-m"),
		newBucket("test-bucket-n"),
		newBucket("test-bucket-o"),
		newBucket("test-bucket-p"),
		newBucket("test-bucket-q"),
		newBucket("test-bucket-r"),
		newBucket("test-bucket-s"),
		newBucket("test-bucket-t"),
		newBucket("test-bucket-u"),
		newBucket("test-bucket-v"),
		newBucket("test-bucket-w"),
		newBucket("test-bucket-x"),
		newBucket("test-bucket-y"),
		newBucket("test-bucket-z"),
	}
}

func hierarchicalObjects() []object {
	return []object{
		{
			objectKey:  "file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "file_2.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/file_2.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/file_3.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/garply/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/waldo/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/fred/file_1.rs",
			objectType: rustCode,
		},
		{
			objectKey:  "foo/quux/fred/file_2.txt",
			objectType: mediumText,
		},
		{
			objectKey:  "foo/quux/fred/plugh/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/fred/xyzzy/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/quux/fred/thud/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/corge/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "foo/grault/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "bar/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "bar/file_2.txt",
			objectType: smallText,
		},
		{
			objectKey:  "baz/file_1.txt",
			objectType: smallText,
		},
		{
			objectKey:  "qux/file_1.txt",
			objectType: smallText,
		},
	}
}

func variousContentTypeObjects() []object {
	return []object{
		{
			objectKey:  "small.txt",
			objectType: smallText,
		},
		{
			objectKey:  "medium.txt",
			objectType: mediumText,
		},
		{
			objectKey:  "large.txt",
			objectType: largeText,
		},
		{
			objectKey:  "hello.html",
			objectType: smallHtml,
		},
		{
			objectKey:       "image.png",
			objectType:      imagePng,
			multipleVersion: true,
		},
		{
			objectKey:       "image.jpeg",
			objectType:      imageJpg,
			multipleVersion: true,
		},
		{
			objectKey:  "empty.txt",
			objectType: empty,
		},
	}
}

func largeNumberObjects() []object {
	size := 3000
	objs := make([]object, size)
	for i := 0; i < size; i++ {
		objs[i] = object{
			objectKey:  fmt.Sprintf("file-%05d.txt", i),
			objectType: smallText,
		}
	}
	return objs
}
