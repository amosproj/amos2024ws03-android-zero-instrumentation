// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl

import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.graphics.drawable.Drawable
import de.amosproj3.ziofa.api.InstalledPackageInfo
import timber.log.Timber

class PackageInformationProvider(private val packageManager: PackageManager) {

    private val installedPackages: Map<String, PackageInfo> by lazy {
        packageManager.getInstalledPackages(PackageManager.GET_META_DATA).associateBy {
            it.packageName
        }
    }

    /**
     * Returns the [InstalledPackageInfo] or null if:
     * - an error occurred
     * - the application info was not found
     * - the package name was not found in the installed packages.
     */
    fun getPackageInfo(packageName: String): InstalledPackageInfo? {
        return try {
            installedPackages[packageName]?.applicationInfo?.let {
                val displayName = packageManager.getApplicationLabel(it).toString()
                val appIcon: Drawable = packageManager.getApplicationIcon(it)
                InstalledPackageInfo(displayName, appIcon)
            }
        } catch (e: Exception) {
            Timber.w(e.stackTraceToString())
            return null
        }
    }
}
