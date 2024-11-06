// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

// Top-level build file where you can add configuration options common to all sub-projects/modules.

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.org.cyclonedx.bom) apply true
    alias(libs.plugins.com.github.benmanes.versions) apply true
    alias(libs.plugins.nl.littlerobots.versioncatalogueupdate) apply true
    alias(libs.plugins.compose.compiler) apply false
    alias(libs.plugins.com.ncorti.ktfmt.gradle) apply true
    alias(libs.plugins.android.library) apply false
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

tasks.cyclonedxBom {
    setIncludeConfigs(listOf("releaseRuntimeClasspath"))
    setProjectType("application")
    setSchemaVersion("1.5")
    setDestination(project.file("build/reports"))
    setOutputName("bom")
    setOutputFormat("json")
    setIncludeBomSerialNumber(false)
    setIncludeLicenseText(true)
    setIncludeMetadataResolution(true)
}

tasks.register("combinedFormat"){
    dependsOn(tasks.ktfmtFormat)
    dependsOn(tasks.versionCatalogFormat)
}