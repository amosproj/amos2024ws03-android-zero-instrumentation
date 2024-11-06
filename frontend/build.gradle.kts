import com.android.utils.TraceUtils.simpleId

// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

// Top-level build file where you can add configuration options common to all sub-projects/modules.

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.org.cyclonedx.bom) apply false
    alias(libs.plugins.com.github.benmanes.versions) apply true
    alias(libs.plugins.nl.littlerobots.versioncatalogueupdate) apply true
    alias(libs.plugins.compose.compiler) apply false
    alias(libs.plugins.com.ncorti.ktfmt.gradle) apply true
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.rust.android) apply false
}


subprojects {
    apply { plugin(rootProject.libs.plugins.com.ncorti.ktfmt.gradle.get().pluginId) }

    ktfmt {
        kotlinLangStyle()
    }
}

fun isNonStable(version: String): Boolean {
    val stableKeyword = listOf("RELEASE", "FINAL", "GA").any { version.uppercase().contains(it) }
    val regex = "^[0-9,.v-]+(-r)?$".toRegex()
    val isStable = stableKeyword || regex.matches(version)
    return isStable.not()
}

tasks.dependencyUpdates.configure {
    rejectVersionIf {
        isNonStable(this.candidate.version)
    }
}

tasks.register("combinedFormat"){
    dependsOn(tasks.ktfmtFormat)
    dependsOn(tasks.versionCatalogFormat)
}