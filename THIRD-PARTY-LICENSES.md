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

## Country name translations

- **Source:** [CLDR](https://cldr.unicode.org/) (Unicode Common Locale Data
  Repository) release 48.2.0
- **Files:** `crates/rs-grid-icons/country_names/*.toml` (14 languages: ar,
  de, es, fr, it, ja, ko, nl, pl, pt, ru, tr, uk, zh)
- **License:** Unicode License V3 (`SPDX: Unicode-3.0`)
- **Copyright:** © 2015-2024 Unicode, Inc.

Territory display names are extracted at build time by
`crates/rs-grid-icons/build.rs` from
`cldr-json/cldr-localenames-full/main/<lang>/territories.json`. The 4
flag-icons UK-subdivision codes (`GB-ENG`/`GB-NIR`/`GB-SCT`/`GB-WLS`) aren't
real CLDR territories and are hand-translated per language instead. The full
license text follows:

```
UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 2015-2024 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.

SPDX-License-Identifier: Unicode-3.0
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
