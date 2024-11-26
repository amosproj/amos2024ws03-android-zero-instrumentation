// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

data class EbpfProgramOptions(val vfsWriteOption: VfsWriteOption)

data class VfsWriteOption(val enabled: Boolean, val pids: List<UInt>)
