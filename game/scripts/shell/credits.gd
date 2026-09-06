class_name CreditsScreen
extends Control

## The one door onto the palette and the type scale (card P11).
const ThemeTokensScript = preload("res://scripts/ui/theme_tokens.gd")

## Attribution for everything this build admitted.
##
## Every line below is copied from a provenance record in the repository and
## names the record it came from. Nothing here is written for the screen: the
## ledgers own these facts, this screen carries them, and
## `game/tests/shell_flow_test.gd` reads the ledgers back off disk and holds the
## two equal, so a licence that changes in the ledger and not here fails the
## gate rather than shipping wrong.
##
## The ledgers cannot be read at runtime: `res://` is `game/`, and
## `content/art/` is not one of the domains the content bundle carries, so the
## records are not in the exported tree. Copying them into `game/` would make a
## second copy that drifts, and folding an `art` domain into the bundle is the
## content lane's call, not the shell's -- so the shell carries the text and the
## suite holds it to the source. When an art domain does reach the bundle, this
## constant is read from it and deleted from here.

signal closed

## Where each block came from, so a reader can check the claim.
const PLACEHOLDER_LEDGER := "content/art/placeholders.json"
const SHARED_LEDGER := "content/art/shared_asset_ledger.json"
const VENDOR_MANIFEST := "work/art/vendor/magnific/betty-3d/N2cYw4m6D9/manifest.json"

## Assets that are in the game tree today, with the provenance their records
## state. Both are stand-ins and both say so: the models are the owner's, and
## nothing in `game/assets/` is a finished asset yet.
const ADMITTED_ASSETS: Array[Dictionary] = [
	{
		"asset": "game/assets/standins/reception_terrace/reception_terrace_backdrop_v1.png",
		"record": "art.placeholder.reception_road.stage",
		"ledger": PLACEHOLDER_LEDGER,
		"field": "source",
		"text": "Built-in image generation, 2026-09-02. Generated from the approved Reception Terrace composition contract; no reference art was copied into the runtime asset."
	},
	{
		"asset": "game/assets/candidates/betty_3d/betty_candidate_v1.glb",
		"record": "art.vendor.magnific.betty_3d.candidate_01",
		"ledger": VENDOR_MANIFEST,
		"field": "source",
		"text": "Magnific, generated with Tripo 3D, downloaded 2026-09-02. https://www.magnific.com/app/3d-scene/N2cYw4m6D9/invite"
	}
]

## The shared source library. Every record is a whole source pack, every one is
## still a candidate, and no member of any of them has been admitted to the
## runtime tree -- so they are credited as what they are.
const SHARED_SOURCES: Array[Dictionary] = [
	{"record": "asset.shared.kenney.nature-kit.2_1.source", "title": "Nature Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/nature-kit"},
	{"record": "asset.shared.kenney.building-kit.source", "title": "Building Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/building-kit"},
	{"record": "asset.shared.kenney.survival-kit.source", "title": "Survival Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/survival-kit"},
	{"record": "asset.shared.kenney.modular-cave-kit.1_0.source", "title": "Modular Cave Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/modular-cave-kit"},
	{"record": "asset.shared.kenney.modular-dungeon-kit.1_0.source", "title": "Modular Dungeon Kit", "creator": "Kenney", "license": "CC0-1.0", "url": "https://kenney.nl/assets/modular-dungeon-kit"}
]

## The ledger's own licence rule, quoted, because it is the reason the list
## above is as short as it is.
const SHARED_POLICY := "CC0-1.0 only. Other sources require project-only or access-controlled treatment unless their exact license is independently recorded as redistribution-permitted."

## Every third-party component this build links, runs on, or is packaged by,
## from the ledger that admits them. E12 gave the engine and the bindings the
## record P8 said they did not have. A component's `notice` names the licence
## body below that it is credited under; an empty `notice` means the ledger
## carries `noticeText: null` for it, because that component's licence text can
## be read neither from a file here nor from its own canonical file upstream --
## so this page says its notice is pending rather than writing one for it.
const THIRD_PARTY_LEDGER := "content/art/third_party_ledger.json"

const THIRD_PARTY: Array[Dictionary] = [
	{"record": "third_party.crate.godot", "name": "godot", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": "mpl_2_0_gdext"},
	{"record": "third_party.crate.godot_core", "name": "godot-core", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": "mpl_2_0_gdext"},
	{"record": "third_party.crate.godot_cell", "name": "godot-cell", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": "mpl_2_0_gdext"},
	{"record": "third_party.crate.godot_ffi", "name": "godot-ffi", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": "mpl_2_0_gdext"},
	{"record": "third_party.crate.godot_macros", "name": "godot-macros", "version": "0.5.5", "license": "MPL-2.0", "url": "https://github.com/godot-rust/gdext", "role": "linked_into_the_gdextension", "notice": "mpl_2_0_gdext"},
	{"record": "third_party.crate.glam", "name": "glam", "version": "0.32.1", "license": "MIT OR Apache-2.0", "url": "https://github.com/bitshifter/glam-rs", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.libc", "name": "libc", "version": "0.2.189", "license": "MIT OR Apache-2.0", "url": "https://github.com/rust-lang/libc", "role": "linked_into_the_gdextension", "notice": "mit_rust_project_developers"},
	{"record": "third_party.crate.proc_macro2", "name": "proc-macro2", "version": "1.0.107", "license": "MIT OR Apache-2.0", "url": "https://github.com/dtolnay/proc-macro2", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.quote", "name": "quote", "version": "1.0.47", "license": "MIT OR Apache-2.0", "url": "https://github.com/dtolnay/quote", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.unicode_ident", "name": "unicode-ident", "version": "1.0.24", "license": "(MIT OR Apache-2.0) AND Unicode-3.0", "url": "https://github.com/dtolnay/unicode-ident", "role": "linked_into_the_gdextension", "notice": "mit_and_unicode_3_0"},
	{"record": "third_party.crate.venial", "name": "venial", "version": "0.6.1", "license": "MIT", "url": "https://github.com/PoignardAzur/venial", "role": "linked_into_the_gdextension", "notice": "mit_olivier_faure"},
	{"record": "third_party.crate.serde", "name": "serde", "version": "1.0.229", "license": "MIT OR Apache-2.0", "url": "https://github.com/serde-rs/serde", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.serde_core", "name": "serde_core", "version": "1.0.229", "license": "MIT OR Apache-2.0", "url": "https://github.com/serde-rs/serde", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.serde_derive", "name": "serde_derive", "version": "1.0.229", "license": "MIT OR Apache-2.0", "url": "https://github.com/serde-rs/serde", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.syn", "name": "syn", "version": "3.0.4", "license": "MIT OR Apache-2.0", "url": "https://github.com/dtolnay/syn", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.serde_json", "name": "serde_json", "version": "1.0.151", "license": "MIT OR Apache-2.0", "url": "https://github.com/serde-rs/json", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.itoa", "name": "itoa", "version": "1.0.18", "license": "MIT OR Apache-2.0", "url": "https://github.com/dtolnay/itoa", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.crate.memchr", "name": "memchr", "version": "2.8.3", "license": "Unlicense OR MIT", "url": "https://github.com/BurntSushi/memchr", "role": "linked_into_the_gdextension", "notice": "mit_andrew_gallant"},
	{"record": "third_party.crate.zmij", "name": "zmij", "version": "1.0.23", "license": "MIT", "url": "https://github.com/dtolnay/zmij", "role": "linked_into_the_gdextension", "notice": "mit_permission_only"},
	{"record": "third_party.engine.godot", "name": "Godot Engine", "version": "4.7.2-stable", "license": "MIT", "url": "https://github.com/godotengine/godot", "role": "runs_the_game", "notice": "mit_godot_engine"},
	{"record": "third_party.engine.godot_export_templates", "name": "Godot Engine export templates", "version": "4.7.2-stable", "license": "MIT", "url": "https://github.com/godotengine/godot", "role": "packaged_into_every_exported_build", "notice": "mit_godot_engine"},
	{"record": "third_party.ci.mesa", "name": "Mesa (mesa-vulkan-drivers, lavapipe)", "version": "", "license": "MIT", "url": "https://gitlab.freedesktop.org/mesa/mesa", "role": "continuous_integration_only", "notice": ""}
]


const NOTICE_BODIES := {
	"mit_permission_only": """Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.""",
	"mit_rust_project_developers": """Copyright (c) The Rust Project Developers

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.""",
	"mit_and_unicode_3_0": """Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.

UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2023 Unicode, Inc.

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
authorization of the copyright holder.""",
	"mit_olivier_faure": """MIT License

Copyright (c) 2022 Olivier FAURE

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.""",
	"mit_andrew_gallant": """The MIT License (MIT)

Copyright (c) 2015 Andrew Gallant

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.""",
	"mpl_2_0_gdext": """Mozilla Public License Version 2.0
==================================

1. Definitions
--------------

1.1. "Contributor"
    means each individual or legal entity that creates, contributes to
    the creation of, or owns Covered Software.

1.2. "Contributor Version"
    means the combination of the Contributions of others (if any) used
    by a Contributor and that particular Contributor's Contribution.

1.3. "Contribution"
    means Covered Software of a particular Contributor.

1.4. "Covered Software"
    means Source Code Form to which the initial Contributor has attached
    the notice in Exhibit A, the Executable Form of such Source Code
    Form, and Modifications of such Source Code Form, in each case
    including portions thereof.

1.5. "Incompatible With Secondary Licenses"
    means

    (a) that the initial Contributor has attached the notice described
        in Exhibit B to the Covered Software; or

    (b) that the Covered Software was made available under the terms of
        version 1.1 or earlier of the License, but not also under the
        terms of a Secondary License.

1.6. "Executable Form"
    means any form of the work other than Source Code Form.

1.7. "Larger Work"
    means a work that combines Covered Software with other material, in 
    a separate file or files, that is not Covered Software.

1.8. "License"
    means this document.

1.9. "Licensable"
    means having the right to grant, to the maximum extent possible,
    whether at the time of the initial grant or subsequently, any and
    all of the rights conveyed by this License.

1.10. "Modifications"
    means any of the following:

    (a) any file in Source Code Form that results from an addition to,
        deletion from, or modification of the contents of Covered
        Software; or

    (b) any new file in Source Code Form that contains any Covered
        Software.

1.11. "Patent Claims" of a Contributor
    means any patent claim(s), including without limitation, method,
    process, and apparatus claims, in any patent Licensable by such
    Contributor that would be infringed, but for the grant of the
    License, by the making, using, selling, offering for sale, having
    made, import, or transfer of either its Contributions or its
    Contributor Version.

1.12. "Secondary License"
    means either the GNU General Public License, Version 2.0, the GNU
    Lesser General Public License, Version 2.1, the GNU Affero General
    Public License, Version 3.0, or any later versions of those
    licenses.

1.13. "Source Code Form"
    means the form of the work preferred for making modifications.

1.14. "You" (or "Your")
    means an individual or a legal entity exercising rights under this
    License. For legal entities, "You" includes any entity that
    controls, is controlled by, or is under common control with You. For
    purposes of this definition, "control" means (a) the power, direct
    or indirect, to cause the direction or management of such entity,
    whether by contract or otherwise, or (b) ownership of more than
    fifty percent (50%) of the outstanding shares or beneficial
    ownership of such entity.

2. License Grants and Conditions
--------------------------------

2.1. Grants

Each Contributor hereby grants You a world-wide, royalty-free,
non-exclusive license:

(a) under intellectual property rights (other than patent or trademark)
    Licensable by such Contributor to use, reproduce, make available,
    modify, display, perform, distribute, and otherwise exploit its
    Contributions, either on an unmodified basis, with Modifications, or
    as part of a Larger Work; and

(b) under Patent Claims of such Contributor to make, use, sell, offer
    for sale, have made, import, and otherwise transfer either its
    Contributions or its Contributor Version.

2.2. Effective Date

The licenses granted in Section 2.1 with respect to any Contribution
become effective for each Contribution on the date the Contributor first
distributes such Contribution.

2.3. Limitations on Grant Scope

The licenses granted in this Section 2 are the only rights granted under
this License. No additional rights or licenses will be implied from the
distribution or licensing of Covered Software under this License.
Notwithstanding Section 2.1(b) above, no patent license is granted by a
Contributor:

(a) for any code that a Contributor has removed from Covered Software;
    or

(b) for infringements caused by: (i) Your and any other third party's
    modifications of Covered Software, or (ii) the combination of its
    Contributions with other software (except as part of its Contributor
    Version); or

(c) under Patent Claims infringed by Covered Software in the absence of
    its Contributions.

This License does not grant any rights in the trademarks, service marks,
or logos of any Contributor (except as may be necessary to comply with
the notice requirements in Section 3.4).

2.4. Subsequent Licenses

No Contributor makes additional grants as a result of Your choice to
distribute the Covered Software under a subsequent version of this
License (see Section 10.2) or under the terms of a Secondary License (if
permitted under the terms of Section 3.3).

2.5. Representation

Each Contributor represents that the Contributor believes its
Contributions are its original creation(s) or it has sufficient rights
to grant the rights to its Contributions conveyed by this License.

2.6. Fair Use

This License is not intended to limit any rights You have under
applicable copyright doctrines of fair use, fair dealing, or other
equivalents.

2.7. Conditions

Sections 3.1, 3.2, 3.3, and 3.4 are conditions of the licenses granted
in Section 2.1.

3. Responsibilities
-------------------

3.1. Distribution of Source Form

All distribution of Covered Software in Source Code Form, including any
Modifications that You create or to which You contribute, must be under
the terms of this License. You must inform recipients that the Source
Code Form of the Covered Software is governed by the terms of this
License, and how they can obtain a copy of this License. You may not
attempt to alter or restrict the recipients' rights in the Source Code
Form.

3.2. Distribution of Executable Form

If You distribute Covered Software in Executable Form then:

(a) such Covered Software must also be made available in Source Code
    Form, as described in Section 3.1, and You must inform recipients of
    the Executable Form how they can obtain a copy of such Source Code
    Form by reasonable means in a timely manner, at a charge no more
    than the cost of distribution to the recipient; and

(b) You may distribute such Executable Form under the terms of this
    License, or sublicense it under different terms, provided that the
    license for the Executable Form does not attempt to limit or alter
    the recipients' rights in the Source Code Form under this License.

3.3. Distribution of a Larger Work

You may create and distribute a Larger Work under terms of Your choice,
provided that You also comply with the requirements of this License for
the Covered Software. If the Larger Work is a combination of Covered
Software with a work governed by one or more Secondary Licenses, and the
Covered Software is not Incompatible With Secondary Licenses, this
License permits You to additionally distribute such Covered Software
under the terms of such Secondary License(s), so that the recipient of
the Larger Work may, at their option, further distribute the Covered
Software under the terms of either this License or such Secondary
License(s).

3.4. Notices

You may not remove or alter the substance of any license notices
(including copyright notices, patent notices, disclaimers of warranty,
or limitations of liability) contained within the Source Code Form of
the Covered Software, except that You may alter any license notices to
the extent required to remedy known factual inaccuracies.

3.5. Application of Additional Terms

You may choose to offer, and to charge a fee for, warranty, support,
indemnity or liability obligations to one or more recipients of Covered
Software. However, You may do so only on Your own behalf, and not on
behalf of any Contributor. You must make it absolutely clear that any
such warranty, support, indemnity, or liability obligation is offered by
You alone, and You hereby agree to indemnify every Contributor for any
liability incurred by such Contributor as a result of warranty, support,
indemnity or liability terms You offer. You may include additional
disclaimers of warranty and limitations of liability specific to any
jurisdiction.

4. Inability to Comply Due to Statute or Regulation
---------------------------------------------------

If it is impossible for You to comply with any of the terms of this
License with respect to some or all of the Covered Software due to
statute, judicial order, or regulation then You must: (a) comply with
the terms of this License to the maximum extent possible; and (b)
describe the limitations and the code they affect. Such description must
be placed in a text file included with all distributions of the Covered
Software under this License. Except to the extent prohibited by statute
or regulation, such description must be sufficiently detailed for a
recipient of ordinary skill to be able to understand it.

5. Termination
--------------

5.1. The rights granted under this License will terminate automatically
if You fail to comply with any of its terms. However, if You become
compliant, then the rights granted under this License from a particular
Contributor are reinstated (a) provisionally, unless and until such
Contributor explicitly and finally terminates Your grants, and (b) on an
ongoing basis, if such Contributor fails to notify You of the
non-compliance by some reasonable means prior to 60 days after You have
come back into compliance. Moreover, Your grants from a particular
Contributor are reinstated on an ongoing basis if such Contributor
notifies You of the non-compliance by some reasonable means, this is the
first time You have received notice of non-compliance with this License
from such Contributor, and You become compliant prior to 30 days after
Your receipt of the notice.

5.2. If You initiate litigation against any entity by asserting a patent
infringement claim (excluding declaratory judgment actions,
counter-claims, and cross-claims) alleging that a Contributor Version
directly or indirectly infringes any patent, then the rights granted to
You by any and all Contributors for the Covered Software under Section
2.1 of this License shall terminate.

5.3. In the event of termination under Sections 5.1 or 5.2 above, all
end user license agreements (excluding distributors and resellers) which
have been validly granted by You or Your distributors under this License
prior to termination shall survive termination.

************************************************************************
*                                                                      *
*  6. Disclaimer of Warranty                                           *
*  -------------------------                                           *
*                                                                      *
*  Covered Software is provided under this License on an "as is"       *
*  basis, without warranty of any kind, either expressed, implied, or  *
*  statutory, including, without limitation, warranties that the       *
*  Covered Software is free of defects, merchantable, fit for a        *
*  particular purpose or non-infringing. The entire risk as to the     *
*  quality and performance of the Covered Software is with You.        *
*  Should any Covered Software prove defective in any respect, You     *
*  (not any Contributor) assume the cost of any necessary servicing,   *
*  repair, or correction. This disclaimer of warranty constitutes an   *
*  essential part of this License. No use of any Covered Software is   *
*  authorized under this License except under this disclaimer.         *
*                                                                      *
************************************************************************

************************************************************************
*                                                                      *
*  7. Limitation of Liability                                          *
*  --------------------------                                          *
*                                                                      *
*  Under no circumstances and under no legal theory, whether tort      *
*  (including negligence), contract, or otherwise, shall any           *
*  Contributor, or anyone who distributes Covered Software as          *
*  permitted above, be liable to You for any direct, indirect,         *
*  special, incidental, or consequential damages of any character      *
*  including, without limitation, damages for lost profits, loss of    *
*  goodwill, work stoppage, computer failure or malfunction, or any    *
*  and all other commercial damages or losses, even if such party      *
*  shall have been informed of the possibility of such damages. This   *
*  limitation of liability shall not apply to liability for death or   *
*  personal injury resulting from such party's negligence to the       *
*  extent applicable law prohibits such limitation. Some               *
*  jurisdictions do not allow the exclusion or limitation of           *
*  incidental or consequential damages, so this exclusion and          *
*  limitation may not apply to You.                                    *
*                                                                      *
************************************************************************

8. Litigation
-------------

Any litigation relating to this License may be brought only in the
courts of a jurisdiction where the defendant maintains its principal
place of business and such litigation shall be governed by laws of that
jurisdiction, without reference to its conflict-of-law provisions.
Nothing in this Section shall prevent a party's ability to bring
cross-claims or counter-claims.

9. Miscellaneous
----------------

This License represents the complete agreement concerning the subject
matter hereof. If any provision of this License is held to be
unenforceable, such provision shall be reformed only to the extent
necessary to make it enforceable. Any law or regulation which provides
that the language of a contract shall be construed against the drafter
shall not be used to construe this License against a Contributor.

10. Versions of the License
---------------------------

10.1. New Versions

Mozilla Foundation is the license steward. Except as provided in Section
10.3, no one other than the license steward has the right to modify or
publish new versions of this License. Each version will be given a
distinguishing version number.

10.2. Effect of New Versions

You may distribute the Covered Software under the terms of the version
of the License under which You originally received the Covered Software,
or under the terms of any subsequent version published by the license
steward.

10.3. Modified Versions

If you create software not governed by this License, and you want to
create a new license for such software, you may create and use a
modified version of this License if you rename the license and remove
any references to the name of the license steward (except to note that
such modified license differs from this License).

10.4. Distributing Source Code Form that is Incompatible With Secondary
Licenses

If You choose to distribute Source Code Form that is Incompatible With
Secondary Licenses under the terms of this version of the License, the
notice described in Exhibit B of this License must be attached.

Exhibit A - Source Code Form License Notice
-------------------------------------------

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at https://mozilla.org/MPL/2.0/.

If it is not possible or desirable to put the notice in a particular
file, then You may include the notice in a location (such as a LICENSE
file in a relevant directory) where a recipient would be likely to look
for such a notice.

You may add additional accurate notices of copyright ownership.

Exhibit B - "Incompatible With Secondary Licenses" Notice
---------------------------------------------------------

  This Source Code Form is "Incompatible With Secondary Licenses", as
  defined by the Mozilla Public License, v. 2.0.""",
	"mit_godot_engine": """Copyright (c) 2014-present Godot Engine contributors (see AUTHORS.md).
Copyright (c) 2007-2014 Juan Linietsky, Ariel Manzur.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.""",
}

## What this build still owes. It is on the credits page rather than in a
## comment because an attribution that is missing is exactly the thing a
## credits page exists to make impossible to forget. P8 owed every third-party
## component a record; what is owed now is narrower and named.
## How many components the ledger carries with no notice text. Held equal to
## the ledger's own `noticePendingCount` by the shell suite, so a notice that
## arrives and is not put on this page fails the gate.
const NOTICES_PENDING := 1

const STILL_OWED := "%d of the components above is credited with the licence its own metadata declares and no notice text: Mesa, whose licence file lives at gitlab.freedesktop.org, a host this repository's build environment cannot reach. Its record names the exact URL and the refusal, so what is left is a fetch and not a search. The notice is pending, not waived."

var close_button: Button


func _init() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)


func _ready() -> void:
	ThemeTokensScript.adopt(self)
	build()


func build() -> void:
	var ground := ColorRect.new()
	ground.name = "Ground"
	ShellStyle.paint(ground, theme, "night")
	ground.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(ground)

	var frame := MarginContainer.new()
	frame.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	frame.add_theme_constant_override("margin_left", 160)
	frame.add_theme_constant_override("margin_right", 160)
	frame.add_theme_constant_override("margin_top", 84)
	frame.add_theme_constant_override("margin_bottom", 64)
	add_child(frame)

	var page := VBoxContainer.new()
	page.add_theme_constant_override("separation", 10)
	frame.add_child(page)

	page.add_child(ShellStyle.label(ShellStyle.tracked("CREDITS"), ShellStyle.DISPLAY))
	page.add_child(ShellStyle.label("Every line here is copied from a provenance record in this repository and names it.", ShellStyle.MUTED))
	page.add_child(ShellStyle.rule(theme, "bronze", 1200.0, 2.0))
	page.add_child(spacer(10))

	var scroll := ScrollContainer.new()
	scroll.name = "CreditsScroll"
	# The notice bodies wrap, and a wrapping label needs a settled width: with
	# horizontal scrolling on, the container would grow sideways to fit the
	# longest line instead and the page would scroll both ways.
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	page.add_child(scroll)
	var body := VBoxContainer.new()
	body.name = "Body"
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 8)
	scroll.add_child(body)

	body.add_child(heading("ASSETS IN THE BUILD"))
	for entry in ADMITTED_ASSETS:
		body.add_child(ShellStyle.label(str(entry["asset"]), ShellStyle.SUBTITLE))
		body.add_child(ShellStyle.label("    %s" % str(entry["text"]), ShellStyle.MUTED))
		body.add_child(ShellStyle.label("    RECORD %s  •  %s" % [str(entry["record"]), str(entry["ledger"])], ShellStyle.FAINT))
		body.add_child(spacer(6))

	body.add_child(heading("SHARED SOURCE LIBRARY  •  ADMITTED AS CANDIDATES, NOT YET IN THE BUILD"))
	for entry in SHARED_SOURCES:
		body.add_child(ShellStyle.label("%s — %s  •  %s  •  %s" % [str(entry["title"]), str(entry["creator"]), str(entry["license"]), str(entry["url"])], ShellStyle.BODY))
	body.add_child(spacer(4))
	body.add_child(ShellStyle.label("    %s" % SHARED_POLICY, ShellStyle.MUTED))
	body.add_child(ShellStyle.label("    LEDGER %s" % SHARED_LEDGER, ShellStyle.FAINT))
	body.add_child(spacer(8))

	body.add_child(heading("THIRD-PARTY COMPONENTS  •  THE ENGINE, THE BINDINGS, AND EVERY CRATE THE EXTENSION LINKS"))
	# One line each, and the line says whether a notice exists. A component with
	# no notice is drawn in the colour the page already uses for what is owed,
	# because that is what it is.
	for component in THIRD_PARTY:
		var version := str(component["version"])
		var titled: String = str(component["name"]) if version == "" else "%s %s" % [str(component["name"]), version]
		var notice := str(component["notice"])
		var standing := str(ROLES.get(str(component["role"]), component["role"]))
		var status := "NOTICE PENDING" if notice == "" else "notice below: %s" % notice
		var tone: StringName = ShellStyle.DANGER if notice == "" else ShellStyle.BODY
		body.add_child(ShellStyle.label("%s  •  %s  •  %s  •  %s  •  %s" % [titled, str(component["license"]), str(component["url"]), standing, status], tone))
	body.add_child(spacer(4))
	body.add_child(ShellStyle.label("    LEDGER %s" % THIRD_PARTY_LEDGER, ShellStyle.FAINT))
	body.add_child(spacer(8))

	body.add_child(heading("LICENCE NOTICES  •  COPIED FROM EACH PROJECT'S OWN LICENCE FILE"))
	for key in NOTICE_BODIES:
		body.add_child(ShellStyle.label(str(key), ShellStyle.BRONZE))
		body.add_child(ShellStyle.label(credited_under(str(key)), ShellStyle.FAINT))
		var notice_label := ShellStyle.label(str(NOTICE_BODIES[key]), ShellStyle.MUTED)
		notice_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		notice_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		body.add_child(notice_label)
		body.add_child(spacer(6))
	body.add_child(spacer(4))

	body.add_child(heading("STILL OWED"))
	body.add_child(ShellStyle.label(STILL_OWED % NOTICES_PENDING, ShellStyle.DANGER))

	page.add_child(spacer(10))
	close_button = Button.new()
	close_button.name = "CloseCredits"
	close_button.text = "BACK"
	close_button.custom_minimum_size = Vector2(240, 48)
	close_button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	close_button.theme_type_variation = ShellStyle.MENU_ROW
	close_button.pressed.connect(close)
	page.add_child(close_button)
	close_button.grab_focus()


## What each `role` means in words, so the page never prints a machine token.
const ROLES := {
	"linked_into_the_gdextension": "linked into the simulation extension",
	"runs_the_game": "the engine this build runs on",
	"packaged_into_every_exported_build": "packaged into every exported build",
	"continuous_integration_only": "used only to render this project's CI captures",
}


## The components credited under one licence body, named so a reader can see
## which notice covers what without counting lines.
func credited_under(key: String) -> String:
	var names: Array[String] = []
	for component in THIRD_PARTY:
		if str(component["notice"]) == key:
			names.append(str(component["name"]))
	return "    covers %s" % ", ".join(names)


## The grammar changed under the page. Every Label and the Back row are Theme
## items and have already moved; the ground and the rule are flat fills.
func _on_theme_rebuilt(rebuilt: Theme) -> void:
	theme = rebuilt
	ShellStyle.repaint_marked(self, rebuilt)


func heading(text: String) -> Label:
	return ShellStyle.label(text, ShellStyle.BRONZE)


func spacer(height: int) -> Control:
	var made := Control.new()
	made.custom_minimum_size = Vector2(0, height)
	made.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return made


func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_cancel"):
		get_viewport().set_input_as_handled()
		close()


func close() -> void:
	closed.emit()
	queue_free()
