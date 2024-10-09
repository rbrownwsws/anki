# WebAssembly addons experiment

This is an experiment in creating Anki addons based on new WebAssembly 
Components standard instead of Python.

For more information about the Webassembly Component Model see:

https://component-model.bytecodealliance.org

## Potential Upsides

### Better safety

Addons run inside a sandbox so it is harder for them to do naughty things.

They should only be able to interact with Anki through the interface we give
them.
No direct system access or ability to monkey patch the Anki application.

This also allows you to implement a permission system a la browser extensions /
mobile apps where the user has control over what the addon can access.

### Mobile/web app support

Addons are run using a Wasm runtime which can be embedded in `rustlib`, no need
for a Python interpreter.

If the app is based on `rustlib` it should be relatively easy to enable addon
support.

You should only need to reimplement the "glue" for each platform, not the
entire addon host architecture.

### Addons can be written in languages other than Python

Many different languages can compile to Wasm Components:

- C/C++
- C#
- Go
- JavaScript
- Python
- Rust

## Potential Downsides

The only downside I can think of using this approach is that the functionality
of addons would be strictly limited to what Anki explicitly exposes.

The creativity of addon authors could be stunted without the "escape hatch" of
directly twiddling with the database, monkey patching etc.

## What is in this directory?

### `./interface/...`

These files define the interface of WebAssembly Anki addons.

They can be used to generate bindings in your preferred language.


### `./examples/...`

These are example addons using the WebAssembly interface.

## How do I try out this experiment?

1. Create a new Anki profile (to make sure we do not mess up your real one).

2. Switch to the new profile.

3. Create an `addons` directory inside your profile directory
   ```
   -- Anki2/
       -- <NORMAL PROFILE>/
           -- ...
       -- <TESTING PROFILE>/
           -- addons/   <--- HERE
           -- backups/
           -- collection.media/
           -- media.trash/
           -- collection.anki2
           -- ...
   ```

4. Build an example addon (see `./examples/rust/README.md`)


5. Put the addon in the `addons` directory
   ```
   -- Anki2/
       -- <TESTING PROFILE>/
           -- addons/
               -- example_addon.wasm  <--- HERE
           -- backups/
           -- ...
   ```

6. Build and run this version of Anki as you normally would (`./run` / `./run.bat`)

   If everything has worked Anki should have started up successfully, and you
   should have a terminal message like this after Anki starts running:

   ```
   [snip]
   
   Starting main loop...
   Qt warning: Remote debugging server started successfully. Try pointing a Chromium-based browser to http://127.0.0.1:8080
   
   Loading addon at: "C:\\Users\\xyz\\AppData\\Roaming\\Anki2\\PluginTesting\\addons\\example_addon.wasm"
   Hello from a Wasm Addon!
   
   [snip]
   ```

7. Marvel at my amazing addon!

   Try adding a note e.g.:

   ```
   Front: hello
    Back: world
   ```

   You should see a terminal message like this:

   ```
   Before add note: DID - 1, Note GUID - FHZ_7FNLjL, Note Fields - ["hello", "world"]
   ```
   
   If you look at the note in the browser the addon should have injected a 
   message into each field of the note.

   ```
   Front: hello [Hello from a Wasm addon!]
    Back: world [Hello from a Wasm addon!]
   ```

   Next have a look at the Tools menu, you should see two new options:

   - `Say Hello`
   - `Say Goodbye`

   If you click these you should get some more messages in the terminal
