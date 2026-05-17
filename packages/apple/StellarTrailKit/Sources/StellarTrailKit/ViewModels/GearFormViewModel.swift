import Foundation
import Combine

@MainActor
final class GearFormViewModel: ObservableObject {
    @Published var draft: CreateGearRequest
    @Published var weightText: String
    @Published var priceText: String
    @Published var tagsText: String
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    @Published private(set) var savedItem: GearItem?

    private let editingID: String?
    private let repository: any GearRepositorying

    init(item: GearItem? = nil, repository: any GearRepositorying) {
        self.editingID = item?.id
        self.repository = repository
        let draft = item.map(CreateGearRequest.init(item:)) ?? .blank
        self.draft = draft
        self.weightText = draft.weightG.map(String.init) ?? ""
        self.priceText = draft.purchasePriceCents.map { String(Double($0) / 100.0) } ?? ""
        self.tagsText = draft.tags?.joined(separator: "，") ?? ""
    }

    var title: String { editingID == nil ? "新增装备" : "编辑装备" }

    func save() async {
        guard draft.name.nilIfBlank != nil else {
            error = "请填写装备名称"
            return
        }
        loading = true
        error = nil
        defer { loading = false }
        do {
            var request = draft
            request.name = draft.name.nilIfBlank ?? draft.name
            request.weightG = Int(weightText.nilIfBlank ?? "")
            if let price = Double(priceText.nilIfBlank ?? "") {
                request.purchasePriceCents = Int((price * 100.0).rounded())
            } else {
                request.purchasePriceCents = nil
            }
            request.tags = tagsText.split(whereSeparator: { "，, ".contains($0) }).map(String.init).filter { !$0.isEmpty }
            if let editingID {
                savedItem = try await repository.update(id: editingID, request: request)
            } else {
                savedItem = try await repository.create(request)
            }
        } catch {
            self.error = error.localizedDescription
        }
    }
}
