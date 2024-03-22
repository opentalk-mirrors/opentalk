// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use gst::prelude::*;
use std::future::Future;

pub trait ElementConvenienceExt: ObjectExt + ElementExt + ElementExtManual {
    fn set_state_async(
        &self,
        state: gst::State,
    ) -> impl Future<Output = Result<gst::StateChangeSuccess>> + '_ {
        self.call_async_future(move |this| {
            this.set_state(state).with_context(|| {
                let name = this.property::<String>("name");
                format!("Failed to set state of {name} to {state:?}")
            })
        })
    }
}

impl<T: ObjectExt + ElementExt + ElementExtManual> ElementConvenienceExt for T {}
