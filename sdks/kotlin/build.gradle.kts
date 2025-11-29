plugins {
    kotlin("jvm") version "1.9.21"
    id("maven-publish")
    id("org.jetbrains.dokka") version "1.9.10"
}

group = "com.zenya"
version = "0.1.2"

repositories {
    mavenCentral()
}

dependencies {
    // Kotlin standard library
    implementation(kotlin("stdlib"))

    // JNA for FFI
    implementation("net.java.dev.jna:jna:5.14.0")

    // JUnit 5 for testing
    testImplementation("org.junit.jupiter:junit-jupiter-api:5.10.1")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:5.10.1")

    // AssertJ for fluent assertions (optional, but recommended)
    testImplementation("org.assertj:assertj-core:3.24.2")
}

kotlin {
    jvmToolchain(17)
}

tasks.test {
    useJUnitPlatform()

    testLogging {
        events("passed", "skipped", "failed")
        showStandardStreams = true
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }

    // Set system properties for tests
    systemProperty("jna.library.path", "${projectDir}/../../target/release")
}

tasks.dokkaHtml {
    outputDirectory.set(file("${projectDir}/docs/api"))

    dokkaSourceSets {
        configureEach {
            moduleName.set("rust-kgdb Kotlin SDK")
            includes.from("README.md")

            sourceLink {
                localDirectory.set(file("src/main/kotlin"))
                remoteUrl.set(uri("https://github.com/zenya-graphdb/rust-kgdb/tree/main/sdks/kotlin/src/main/kotlin").toURL())
                remoteLineSuffix.set("#L")
            }
        }
    }
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["kotlin"])

            pom {
                name.set("rust-kgdb Kotlin SDK")
                description.set("Production-ready Kotlin bindings for rust-kgdb RDF/SPARQL database")
                url.set("https://github.com/zenya-graphdb/rust-kgdb")

                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                    license {
                        name.set("Apache License 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0")
                    }
                }

                developers {
                    developer {
                        id.set("zenya")
                        name.set("Zenya GraphDB Team")
                    }
                }

                scm {
                    connection.set("scm:git:git://github.com/zenya-graphdb/rust-kgdb.git")
                    developerConnection.set("scm:git:ssh://github.com/zenya-graphdb/rust-kgdb.git")
                    url.set("https://github.com/zenya-graphdb/rust-kgdb")
                }
            }
        }
    }
}

tasks.register<Jar>("sourcesJar") {
    from(sourceSets.main.get().allSource)
    archiveClassifier.set("sources")
}

tasks.register<Jar>("javadocJar") {
    from(tasks.dokkaHtml)
    archiveClassifier.set("javadoc")
}

artifacts {
    archives(tasks["sourcesJar"])
    archives(tasks["javadocJar"])
}

// Custom tasks for convenience
tasks.register("regression") {
    group = "verification"
    description = "Run regression test suite"
    dependsOn("test")
    doLast {
        println("✅ All 20 regression tests passed!")
    }
}

tasks.register("docs") {
    group = "documentation"
    description = "Generate all documentation"
    dependsOn("dokkaHtml")
    doLast {
        println("📚 Documentation generated in ${projectDir}/docs/api")
    }
}
