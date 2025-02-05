// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.processes

import android.graphics.drawable.Drawable

/** Wrapper class for app information */
data class InstalledPackageInfo(val displayName: String, val icon: Drawable)
