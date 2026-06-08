# Third-party licenses

rs-grid is licensed under the [MIT License](LICENSE). It bundles a small
number of third-party assets, listed below together with their respective
licenses and attribution.

## Country flags

- **Source:** [flag-icons](https://github.com/lipis/flag-icons) v7.5.0
- **Files:** `crates/rs-grid-icons/flags/*.svg` (254 SVGs, ISO 3166-1 alpha-2)
- **License:** MIT
- **Copyright:** © 2013 Panayiotis Lipiridis

The SVGs are embedded as base64 data URIs at build time by
`crates/rs-grid-icons/build.rs`. The full license text follows:

```
The MIT License (MIT)

Copyright (c) 2013 Panayiotis Lipiridis

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Gender symbols

- **Files:** `crates/rs-grid-icons/genders/male.svg`,
  `crates/rs-grid-icons/genders/female.svg`
- **License:** CC0 1.0 (public domain dedication)

Original minimal renditions of the standard Unicode Mars (♂, U+2642) and
Venus (♀, U+2640) symbols, created for this project. No third-party content.

## DaisyUI theme values

- **Source:** [DaisyUI](https://github.com/saadeghi/daisyui) v5.5.19
  (`daisyui/theme/object.js` + `daisyui/components/*/object.js`)
- **Files:** `examples/example-common/src/class_map_data.rs` — colour and
  geometry constants derived from DaisyUI's light theme, generated at build
  time by `tools/class-map/generate_class_map.mjs`.
- **License:** MIT
- **Copyright:** © Pouya Saadeghi

The values are a derivative of DaisyUI's MIT-licensed theme data; the MIT
notice is retained below as required.

```
MIT License

Copyright (c) Pouya Saadeghi

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
