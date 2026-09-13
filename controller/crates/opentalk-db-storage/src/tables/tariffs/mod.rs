// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains tariffs table structs

mod external_tariff;
mod external_tariff_id;
mod new_tariff;
mod tariff;
mod update_tariff;

pub use external_tariff::ExternalTariff;
pub use external_tariff_id::ExternalTariffId;
pub use new_tariff::NewTariff;
pub use tariff::Tariff;
pub use update_tariff::UpdateTariff;
