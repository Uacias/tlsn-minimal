import SwiftUI

struct ContentView: View {
    @State private var message: String = "Press the button"

    var body: some View {
        VStack {
            Text(message)
                .padding()
            
            Button(action: fetchMessage) {
                Label("Fetch from Rust", systemImage: "arrow.down.circle")
            }
            .padding()
        }
    }

    private func fetchMessage() {
        if let cString = say_hello() {
            let text = String(cString: cString)
            DispatchQueue.main.async {
                message = text
            }
        }
    }
}

#Preview {
    ContentView()
}

