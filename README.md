# Staff

> The staff to conquer the dunge

The CLI-tool for working with graphical objects such as meshes, animations (actions), character skeletons, sprites, etc. I use it to convert models from Blender into a usable way to work with it in my graphics library [dunge](https://github.com/nanoqsh/dunge).

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
