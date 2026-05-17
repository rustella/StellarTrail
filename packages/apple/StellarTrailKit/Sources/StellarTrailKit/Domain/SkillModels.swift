import Foundation

struct SkillCategorySummary: Codable, Equatable, Identifiable {
    let id: String
    let slug: String?
    let title: String
    let summary: String
    let itemCount: Int
    let href: String?
}

struct SkillCategoriesResponse: Codable, Equatable {
    let items: [SkillCategorySummary]
}

struct PageInfo: Codable, Equatable {
    let limit: Int
    let offset: Int
    let nextOffset: Int?
}

struct KnotTaxonomyItem: Codable, Equatable, Identifiable {
    let id: String
    let slug: String?
    let title: String
}

struct KnotMediaAsset: Codable, Equatable, Identifiable {
    let id: String
    let mediaType: String
    let url: String
    let mimeType: String
    let width: Int?
    let height: Int?
    let sizeBytes: Int
    let attribution: String?
    let licenseNote: String?
}

struct KnotSummary: Codable, Equatable, Identifiable {
    let id: String
    let slug: String?
    let title: String
    let summary: String
    let difficulty: String?
    let categories: [KnotTaxonomyItem]
    let types: [KnotTaxonomyItem]
    let media: [KnotMediaAsset]
    let href: String?

    var mediaCount: Int { media.count }

    enum CodingKeys: String, CodingKey {
        case id, slug, title, summary, difficulty, categories, types, media, href
        case mediaCount
    }

    init(id: String, slug: String?, title: String, summary: String, difficulty: String?, categories: [KnotTaxonomyItem], types: [KnotTaxonomyItem], media: [KnotMediaAsset], href: String?) {
        self.id = id
        self.slug = slug
        self.title = title
        self.summary = summary
        self.difficulty = difficulty
        self.categories = categories
        self.types = types
        self.media = media
        self.href = href
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(String.self, forKey: .id)
        slug = try container.decodeIfPresent(String.self, forKey: .slug)
        title = try container.decode(String.self, forKey: .title)
        summary = try container.decode(String.self, forKey: .summary)
        difficulty = try container.decodeIfPresent(String.self, forKey: .difficulty)
        categories = try container.decodeIfPresent([KnotTaxonomyItem].self, forKey: .categories) ?? []
        types = try container.decodeIfPresent([KnotTaxonomyItem].self, forKey: .types) ?? []
        media = try container.decodeIfPresent([KnotMediaAsset].self, forKey: .media) ?? []
        href = try container.decodeIfPresent(String.self, forKey: .href)
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encodeIfPresent(slug, forKey: .slug)
        try container.encode(title, forKey: .title)
        try container.encode(summary, forKey: .summary)
        try container.encodeIfPresent(difficulty, forKey: .difficulty)
        try container.encode(categories, forKey: .categories)
        try container.encode(types, forKey: .types)
        try container.encode(media, forKey: .media)
        try container.encodeIfPresent(href, forKey: .href)
    }
}

struct KnotListResponse: Codable, Equatable {
    let locale: String?
    let items: [KnotSummary]
    let page: PageInfo

    var nextOffset: Int? { page.nextOffset }

    enum CodingKeys: String, CodingKey {
        case locale, items, page
        case nextOffset
    }

    init(locale: String? = "zh-CN", items: [KnotSummary], page: PageInfo) {
        self.locale = locale
        self.items = items
        self.page = page
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        locale = try container.decodeIfPresent(String.self, forKey: .locale)
        items = try container.decode([KnotSummary].self, forKey: .items)
        if let page = try container.decodeIfPresent(PageInfo.self, forKey: .page) {
            self.page = page
        } else {
            let nextOffset = try container.decodeIfPresent(Int.self, forKey: .nextOffset)
            self.page = PageInfo(limit: items.count, offset: 0, nextOffset: nextOffset)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encodeIfPresent(locale, forKey: .locale)
        try container.encode(items, forKey: .items)
        try container.encode(page, forKey: .page)
    }
}

typealias ListKnotsResponse = KnotListResponse

struct KnotDetail: Codable, Equatable, Identifiable {
    let id: String
    let slug: String?
    let title: String
    let summary: String
    let difficulty: String?
    let categories: [KnotTaxonomyItem]
    let types: [KnotTaxonomyItem]
    let media: [KnotMediaAsset]
    let href: String?
    let description: String?
    let steps: [String]
    let locale: String?

    var mediaCount: Int { media.count }
}

struct ListKnotsRequest: Equatable {
    var offset: Int = 0
    var limit: Int = 20
    var category: String?
    var q: String?

    var queryItems: [URLQueryItem] {
        var items = [
            URLQueryItem(name: "offset", value: String(offset)),
            URLQueryItem(name: "limit", value: String(limit))
        ]
        if let category { items.append(URLQueryItem(name: "category", value: category)) }
        if let q = q?.nilIfBlank { items.append(URLQueryItem(name: "q", value: q)) }
        return items
    }
}
