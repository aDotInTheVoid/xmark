// SPDX-License-Identifier: GPL-3.0-only
use insta::internals::Content;

pub fn manifest_dir_redacter(mut val: Content, _: insta::internals::ContentPath) -> Content {
    // TODO: Dont do insta crimes.
    // This is in #[doc(Hidden)] internals.
    while let insta::internals::Content::Some(some) = val {
        val = *some;
    }

    if let insta::internals::Content::String(s) = val {
        s.replace(env!("CARGO_MANIFEST_DIR"), "BASEDIR").into()
    } else {
        val
    }
}
