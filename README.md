# Staff

> The staff to conquer the dunge

The CLI-tool for working with graphical objects such as meshes, animations (actions), character skeletons, sprites, etc. I use it to convert models from Blender into a usable way to work with it in my graphics library [dunge](https://github.com/nanoqsh/dunge), as well to work with sprites.

At the moment, this tool is exclusively for personal use, it is unlikely that you will need it.

## Build and install
The most convenient way to do everything at once:
```
cargo install --locked --git https://github.com/nanoqsh/staff.git
```

You can then call the tool directly:
```
staff
```

## Conversions
Export an object from Blender in collada `.dae` format. In order for the object to have the correct orientation in `dunge`, apply global orientation:
- X for the forward axis.
- Y for the up axis.

For example, I want to save a mesh in `model.dae` file.
```
staff convert mesh model.dae
```

If everything went well, it will write a `.json` file to the working directory. Use the `-o` or `--outdir` flag to specify exactly where you want to save files.

## Sprites repainting
It would be cool to be able to recolor sprites in a desired palette. First you need to collect the palette itself. To do this, specify a `.png` image with specific colors:
```
staff collect palette.png
```

It collect all unique colors and save them to a file. Then, you can recolor the image using this palette. It take the `palette.json` file by default, but you can specify a specific palette file with the following argument:
```
staff repaint sprite.png palette.json
```

## Create a sprite atlas
To create the atlas, you need to specify all the sprites that need to be glued together. For example, let's take all images from the `sprites` directory:
```
staff atlas sprites/*.png
```

This will create two files: `atlas.png` for the final image and `atlas.json` for the sprite map. The sprite map describes all entries by four values: x position, y position, width and height. The sprites names are taken from their file names. To set your own names, you can describe a `.json` file for name conversion, it looks like this:
```json
{
    "colon": ":",
    "dot": ".",
    "dquote": "\"",
    "squote": "'",
    "slash": "/",
    "star": "*"
}
```

To use the transformation, specify this file:
```
staff atlas sprites/*.png --names names.json
```

Specifying the file is optional, otherwise it will default to `names.json` file.

A detailed description of all parameters can be obtained from the help:
```
staff atlas --help
```
