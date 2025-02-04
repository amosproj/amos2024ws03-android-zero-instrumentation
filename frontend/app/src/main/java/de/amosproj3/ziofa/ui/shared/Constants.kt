// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.client.Configuration

/**
 * Empty configuration to set if there is no configuration set (initially) or if the configuration
 * is reset via the ResetScreen
 */
val EMPTY_CONFIGURATION = Configuration(null, null, listOf(), null, null, null, null)
