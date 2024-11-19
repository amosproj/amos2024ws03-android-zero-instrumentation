// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

import uniffi.shared.EbpfEntry

data class EBpfProgramOption(
    val name: String,
    val active: Boolean,
    val confirmed: Boolean, //TODO show diff
    val ebpfEntry: EbpfEntry
)
