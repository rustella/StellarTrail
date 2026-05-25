import Foundation
import Combine

@MainActor
final class GearFormViewModel: ObservableObject {
    @Published var draft: GearFormDraft
    @Published private(set) var specFields: [GearSpecFieldView]
    @Published private(set) var rankedSpecKeys: [String] = []
    @Published private(set) var tagSuggestions: [GearTagSuggestionView] = []
    @Published var tagInput = ""
    @Published var selectedTagColor: GearTagColor = .teal
    @Published private(set) var loading = false
    @Published private(set) var saving = false
    @Published private(set) var error: String?
    @Published private(set) var savedItem: GearItem?

    private let editingID: String?
    private let repository: any GearRepositorying

    init(item: GearItem? = nil, repository: any GearRepositorying) {
        self.editingID = item?.id
        self.repository = repository
        let draft = item.map(GearFormDraft.init(item:)) ?? .blank
        self.draft = draft
        self.specFields = GearOptions.specFieldViews(for: draft.category, specs: draft.specs)
    }

    var title: String { editingID == nil ? "新增装备" : "编辑装备" }

    func loadOptions() async {
        loading = true
        error = nil
        defer { loading = false }
        do {
            async let rankings = repository.specKeyRankings(category: draft.category)
            async let suggestions = repository.tagSuggestions(limit: 20)
            let rankingsResponse = try await rankings
            let suggestionsResponse = try await suggestions
            rankedSpecKeys = rankingsResponse.keys
            tagSuggestions = GearOptions.createTagSuggestionViews(suggestionsResponse.items)
            refreshSpecFields()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func selectCategory(_ category: GearCategory) async {
        draft.category = category
        draft.specs = GearOptions.normalizeSpecs(category: category, specs: draft.specs) ?? [:]
        await loadOptions()
    }

    func updateSpecValue(key: String, value: String, unit: String) {
        draft.specs[key] = GearOptions.combineSpecValue(value, unit: unit)
        refreshSpecFields()
    }

    func updateSpecUnit(key: String, fieldIndex: Int, unitIndex: Int) {
        guard specFields.indices.contains(fieldIndex) else { return }
        let field = specFields[fieldIndex]
        let unit = field.units.indices.contains(unitIndex) ? field.units[unitIndex] : ""
        draft.specs[key] = GearOptions.combineSpecValue(field.valueText, unit: unit)
        refreshSpecFields()
    }

    func addTagsFromInput() {
        draft.tags = GearOptions.addTagViews(current: draft.tags, input: tagInput, color: selectedTagColor)
        tagInput = ""
    }

    func addSuggestion(_ suggestion: GearTagSuggestionView) {
        draft.tags = GearOptions.addTagViews(current: draft.tags, input: suggestion.name, color: suggestion.color)
    }

    func removeTag(_ tag: GearTagView) {
        draft.tags.removeAll { $0.id == tag.id }
    }

    func save() async {
        saving = true
        error = nil
        defer { saving = false }
        do {
            let request = try draft.buildGearPayload()
            if let editingID {
                savedItem = try await repository.update(id: editingID, request: request)
            } else {
                savedItem = try await repository.create(request)
            }
        } catch {
            self.error = error.localizedDescription
        }
    }

    private func refreshSpecFields() {
        specFields = GearOptions.specFieldViews(for: draft.category, specs: draft.specs, rankedKeys: rankedSpecKeys)
    }
}
