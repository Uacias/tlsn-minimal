//
//  Item.swift
//  tlsn_example
//
//  Created by Piotr Stec on 14/03/2025.
//

import Foundation
import SwiftData

@Model
final class Item {
    var timestamp: Date
    
    init(timestamp: Date) {
        self.timestamp = timestamp
    }
}
