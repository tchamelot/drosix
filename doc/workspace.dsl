workspace "Drosix" {
    !docs workspace-docs

    model {
        user = person "User" "Pilotes the drone with a remote controller"
        dev = person "Dev" "Codes, tunes and debug"


        drosix = softwareSystem "Drosix" "Drone" {
            !docs drosix/docs
            params = container "Drosix Parameters" "Stores flight parameters" "TOML" "Storage"
            flightController = container "Flight controller" "Flights the drone" "Rust thread" {
                configurator = component "Configurator" "Stores Drosix parameters" "Rust"
                missionController = component "Mission Controller" "Schedules all components execution" "Rust"
                pruController = component "PRU controller" "Exposes API to PRU subsystems" "Rust"
                sensors = component "Sensors" "Configures and reads sensors" "Rust"
                imu = component "MPU9250" "Inertial measurement unit" "Propietary" "Hardware"
            }
            remotehandler = container "Remote controller" "Handle remote controller inputs" "Rust thread"
            server = container "Backend server" "Serves single page app, exposes REST backend" "Rust thread"
            pru0 = container "PID controller" "Coprocessor computing motors speed command" "C" {
                scheduler0 = component "Scheduler" "Schedule all the events" "C"
                timer0 = component "PID Timer" "Schedule the pid computation at 100Hz" "Hardware" "Hardware"
                pid = component "PID controller" "Compute the PID" "C"
                rate = component "rate controller" "Compute the motor output" "C" 
                attitude = component "Attitude controller" "Compute the desired rate" "C" 
                
            }
            pru1 = container "PWM controller" "Coprocessor computing PWM signals from motor speeds" "C"
            
            imu -> sensors "Notifies" "GPIO interrupt"
            sensors -> imu "Configures, reads data from" "I2C"
            
            remotehandler -> missionController "Sends command" "channel<Command>"
            server -> flightController "Sends command" "channel<Command>"
            
            configurator -> params "Reads from, writes to" "File system"
            
            pruController -> pru0 "Configures, Notifies event, sends data to" "Interrupt, shared memory"
            pruController -> pru1 "Configures" "Shared memory"
            scheduler0 -> pruController "Notifies event" "Interrupt"
            scheduler0 -> pru1 "Notifies event" "Interrupt"
            pru1 -> pru0 "Notifies event" "Interrupt"
            
            missionController -> configurator "Gets / sets parameters" "API"
            missionController -> sensors "Reads data" "async API"
            missionController -> pruController "Uses" "API"

            timer0 -> scheduler0 "Notifies" "Interrupt"
            scheduler0 -> timer0 "Configures" "API"
            scheduler0 -> rate "Calls" "API"
            rate -> attitude "Calls" "API"
            rate -> pid "Calls" "API"
            rate -> pru1 "Sends motors command" "Interrupt, shared memory"
            attitude -> pid "Calls" "API"


        }

        
        user -> remotehandler "Sends command to" "bluetooth"
        dev -> server "Sends command to"
        server -> dev "Sends log to"
    }

    views {
        properties {
            "c4plantuml.elementProperties" "true"
            "c4plantuml.tags" "true"
            "generatr.style.colors.primary" "#485fc7"
            "generatr.style.colors.secondary" "#ffffff"
            "generatr.style.faviconPath" "site/favicon.ico"
            "generatr.style.logoPath" "site/drosix.jpg"

            // Absolute URL's like "https://example.com/custom.css" are also supported
            "generatr.style.customStylesheet" "site/custom.css"

            "generatr.svglink.target" "_self"

            "generatr.markdown.flexmark.extensions" "Abbreviation,Admonition,AnchorLink,Attributes,Autolink,Definition,Emoji,Footnotes,GfmTaskList,GitLab,MediaTags,Tables,TableOfContents,Typographic"

            "generatr.site.exporter" "c4"
            "generatr.site.externalTag" "External System"
            "generatr.site.nestGroups" "false"
        }

        systemContext "drosix" "SystemContext" {
            default
            include *
            autoLayout
        }
        
        container drosix "Containers" {
            include *
            autoLayout
        }
        
        component flightController "FlightController" {
            include *
            exclude "pru0 -> pru1"
            exclude "pru1 -> pru0"
            autoLayout
        }
        
        component pru0 "PRU0" {
            include *
            autoLayout
        }

        styles {
            element "External" {
                background #999999
            }

            element "Hardware" {
                background #f08585
            }
            
            element "Storage" {
                shape Cylinder
            }
        }
        theme default
    }
    
}
