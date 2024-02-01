// btleplug Source Code File
//
// Copyright 2020 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.
//
// Some portions of this file are taken and/or modified from Rumble
// (https://github.com/mwylde/rumble), using a dual MIT/Apache License under the
// following copyright:
//
// Copyright (c) 2014 The Rust Project Developers

use crate::{api::ScanFilter, Error, Result};
use windows::{Devices::Bluetooth::Advertisement::*, Foundation::TypedEventHandler};

pub type AdvertismentEventHandler = Box<dyn Fn(&BluetoothLEAdvertisementReceivedEventArgs) + Send>;

pub struct BLEWatcher {
    watcher: BluetoothLEAdvertisementWatcher,
}

impl From<windows::core::Error> for Error {
    fn from(err: windows::core::Error) -> Error {
        println!("Error occurred: {:?}", err);
        Error::Other(format!("{:?}", err).into())
    }
}

impl BLEWatcher {
    pub fn new() -> Self {
        println!("Creating new BLEWatcher");
        let ad = BluetoothLEAdvertisementFilter::new().unwrap();
        let watcher = BluetoothLEAdvertisementWatcher::Create(&ad).unwrap();
        println!("BLEWatcher created successfully.");
        BLEWatcher { watcher }
    }

    pub fn start(&self, filter: ScanFilter, on_received: AdvertismentEventHandler) -> Result<()> {
        println!("Starting BLEWatcher");
        let ScanFilter { services } = filter;
        let ad = self
            .watcher
            .AdvertisementFilter()
            .unwrap()
            .Advertisement()
            .unwrap();
        println!("Advertisement: {:?}", ad);

        let ad_services = ad.ServiceUuids().unwrap();
        println!("Existing services: {:?}", ad_services);
        ad_services.Clear().unwrap();
        for service in services {
            println!("Appending service: {:?}", service);
            ad_services
                .Append(windows::core::GUID::from(service.as_u128()))
                .unwrap();
        }
        println!("Updated services: {:?}", ad_services);
        self.watcher
            .SetScanningMode(BluetoothLEScanningMode::Active)
            .unwrap();
        self.watcher.SetAllowExtendedAdvertisements(true)?;
        let handler: TypedEventHandler<
            BluetoothLEAdvertisementWatcher,
            BluetoothLEAdvertisementReceivedEventArgs,
        > = TypedEventHandler::new(
            move |_sender, args: &Option<BluetoothLEAdvertisementReceivedEventArgs>| {
                println!("Advertisement received");
                if let Some(args) = args {
                    on_received(args);
                }
                Ok(())
            },
        );

        println!("Setting Received handler");
        self.watcher.Received(&handler)?;
        println!("Starting Advertisment watcher");
        self.watcher.Start()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        println!("Stopping BLEWatcher");
        self.watcher.Stop()?;
        println!("BLEWatcher stopped.");
        Ok(())
    }
}
