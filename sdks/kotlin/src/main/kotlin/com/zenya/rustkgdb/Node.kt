package com.zenya.rustkgdb

import uniffi.gonnect.GonnectNode

/**
 * RDF node (IRI, Literal, or Blank Node).
 *
 * This class provides factory methods for creating different types of RDF nodes
 * with an intuitive, type-safe API.
 *
 * ## Node Types
 *
 * - **IRI**: Named resources identified by URI
 * - **Literal**: String, numeric, or language-tagged values
 * - **Blank Node**: Anonymous nodes
 *
 * ## Quick Examples
 *
 * ```kotlin
 * // IRIs
 * val person = Node.iri("http://example.org/alice")
 * val nameProperty = Node.iri("http://xmlns.com/foaf/0.1/name")
 *
 * // Literals
 * val name = Node.literal("Alice")
 * val age = Node.integer(30)
 * val active = Node.boolean(true)
 * val title = Node.langLiteral("Bonjour", "fr")
 *
 * // Blank nodes
 * val anonymous = Node.blank("b1")
 * ```
 *
 * @see GraphDB for using nodes in queries and updates
 */
class Node private constructor(private val ffiNode: GonnectNode) {

    companion object {
        /**
         * Creates an IRI node.
         *
         * IRIs (Internationalized Resource Identifiers) are used to identify
         * resources and properties in RDF.
         *
         * @param uri The full URI string (e.g., "http://example.org/resource")
         * @return A Node representing the IRI
         *
         * ## Example
         *
         * ```kotlin
         * val alice = Node.iri("http://example.org/alice")
         * val foafName = Node.iri("http://xmlns.com/foaf/0.1/name")
         * val rdfType = Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
         * ```
         */
        @JvmStatic
        fun iri(uri: String): Node {
            return Node(GonnectNode.Iri(uri))
        }

        /**
         * Creates a plain literal node.
         *
         * Plain literals are string values without datatype or language tags.
         *
         * @param value The string value
         * @return A Node representing the literal
         *
         * ## Example
         *
         * ```kotlin
         * val name = Node.literal("Alice")
         * val description = Node.literal("A person")
         * ```
         */
        @JvmStatic
        fun literal(value: String): Node {
            return Node(GonnectNode.Literal(value, null, null))
        }

        /**
         * Creates a typed literal node.
         *
         * Typed literals have an explicit XSD datatype.
         *
         * @param value The string representation of the value
         * @param datatype The datatype IRI (e.g., "http://www.w3.org/2001/XMLSchema#integer")
         * @return A Node representing the typed literal
         *
         * ## Example
         *
         * ```kotlin
         * val xsdInteger = "http://www.w3.org/2001/XMLSchema#integer"
         * val age = Node.typedLiteral("30", xsdInteger)
         * ```
         */
        @JvmStatic
        fun typedLiteral(value: String, datatype: String): Node {
            return Node(GonnectNode.Literal(value, datatype, null))
        }

        /**
         * Creates a language-tagged literal node.
         *
         * Language-tagged literals specify the natural language of the text value.
         *
         * @param value The string value
         * @param lang The language tag (e.g., "en", "fr", "de")
         * @return A Node representing the language-tagged literal
         *
         * ## Example
         *
         * ```kotlin
         * val englishName = Node.langLiteral("Hello", "en")
         * val frenchName = Node.langLiteral("Bonjour", "fr")
         * val germanName = Node.langLiteral("Guten Tag", "de")
         * ```
         */
        @JvmStatic
        fun langLiteral(value: String, lang: String): Node {
            return Node(GonnectNode.Literal(value, null, lang))
        }

        /**
         * Creates an integer literal node.
         *
         * Convenience method for creating XSD integer literals.
         *
         * @param value The integer value
         * @return A Node representing the integer
         *
         * ## Example
         *
         * ```kotlin
         * val age = Node.integer(30)
         * val count = Node.integer(42)
         * ```
         */
        @JvmStatic
        fun integer(value: Int): Node {
            return Node(GonnectNode.Literal(
                value.toString(),
                "http://www.w3.org/2001/XMLSchema#integer",
                null
            ))
        }

        /**
         * Creates a long integer literal node.
         *
         * @param value The long value
         * @return A Node representing the long integer
         */
        @JvmStatic
        fun long(value: Long): Node {
            return Node(GonnectNode.Literal(
                value.toString(),
                "http://www.w3.org/2001/XMLSchema#long",
                null
            ))
        }

        /**
         * Creates a double literal node.
         *
         * @param value The double value
         * @return A Node representing the double
         *
         * ## Example
         *
         * ```kotlin
         * val price = Node.double(19.99)
         * val temperature = Node.double(98.6)
         * ```
         */
        @JvmStatic
        fun double(value: Double): Node {
            return Node(GonnectNode.Literal(
                value.toString(),
                "http://www.w3.org/2001/XMLSchema#double",
                null
            ))
        }

        /**
         * Creates a float literal node.
         *
         * @param value The float value
         * @return A Node representing the float
         */
        @JvmStatic
        fun float(value: Float): Node {
            return Node(GonnectNode.Literal(
                value.toString(),
                "http://www.w3.org/2001/XMLSchema#float",
                null
            ))
        }

        /**
         * Creates a boolean literal node.
         *
         * Convenience method for creating XSD boolean literals.
         *
         * @param value The boolean value
         * @return A Node representing the boolean
         *
         * ## Example
         *
         * ```kotlin
         * val active = Node.boolean(true)
         * val deleted = Node.boolean(false)
         * ```
         */
        @JvmStatic
        fun boolean(value: Boolean): Node {
            return Node(GonnectNode.Literal(
                value.toString(),
                "http://www.w3.org/2001/XMLSchema#boolean",
                null
            ))
        }

        /**
         * Creates a date literal node.
         *
         * @param value The date string in ISO 8601 format (YYYY-MM-DD)
         * @return A Node representing the date
         *
         * ## Example
         *
         * ```kotlin
         * val birthDate = Node.date("1990-05-15")
         * val today = Node.date("2025-11-28")
         * ```
         */
        @JvmStatic
        fun date(value: String): Node {
            return Node(GonnectNode.Literal(
                value,
                "http://www.w3.org/2001/XMLSchema#date",
                null
            ))
        }

        /**
         * Creates a dateTime literal node.
         *
         * @param value The dateTime string in ISO 8601 format
         * @return A Node representing the dateTime
         *
         * ## Example
         *
         * ```kotlin
         * val timestamp = Node.dateTime("2025-11-28T22:15:00Z")
         * ```
         */
        @JvmStatic
        fun dateTime(value: String): Node {
            return Node(GonnectNode.Literal(
                value,
                "http://www.w3.org/2001/XMLSchema#dateTime",
                null
            ))
        }

        /**
         * Creates a blank node.
         *
         * Blank nodes are anonymous nodes without global identifiers.
         * They are scoped to the current document.
         *
         * @param id The local identifier (e.g., "b1", "person1")
         * @return A Node representing the blank node
         *
         * ## Example
         *
         * ```kotlin
         * val anonymous = Node.blank("b1")
         * val person = Node.blank("person1")
         * ```
         */
        @JvmStatic
        fun blank(id: String): Node {
            return Node(GonnectNode.BlankNode(id))
        }
    }

    /**
     * Converts this high-level Node to the FFI layer representation.
     *
     * @return The underlying FFI node
     */
    internal fun toFFI(): GonnectNode = ffiNode

    override fun toString(): String {
        return when (ffiNode) {
            is GonnectNode.Iri -> "<${ffiNode.value}>"
            is GonnectNode.Literal -> {
                val value = ffiNode.value
                val datatype = ffiNode.datatype
                val lang = ffiNode.language
                when {
                    lang != null -> "\"$value\"@$lang"
                    datatype != null -> "\"$value\"^^<$datatype>"
                    else -> "\"$value\""
                }
            }
            is GonnectNode.BlankNode -> "_:${ffiNode.id}"
            else -> ffiNode.toString()
        }
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Node) return false
        return ffiNode == other.ffiNode
    }

    override fun hashCode(): Int = ffiNode.hashCode()
}

/**
 * Common XSD datatypes as constants for convenience.
 *
 * ## Example
 *
 * ```kotlin
 * import com.zenya.rustkgdb.XSD
 *
 * val age = Node.typedLiteral("30", XSD.INTEGER)
 * val price = Node.typedLiteral("19.99", XSD.DECIMAL)
 * ```
 */
object XSD {
    /** XSD string datatype */
    const val STRING = "http://www.w3.org/2001/XMLSchema#string"

    /** XSD integer datatype */
    const val INTEGER = "http://www.w3.org/2001/XMLSchema#integer"

    /** XSD long datatype */
    const val LONG = "http://www.w3.org/2001/XMLSchema#long"

    /** XSD int datatype */
    const val INT = "http://www.w3.org/2001/XMLSchema#int"

    /** XSD short datatype */
    const val SHORT = "http://www.w3.org/2001/XMLSchema#short"

    /** XSD byte datatype */
    const val BYTE = "http://www.w3.org/2001/XMLSchema#byte"

    /** XSD decimal datatype */
    const val DECIMAL = "http://www.w3.org/2001/XMLSchema#decimal"

    /** XSD double datatype */
    const val DOUBLE = "http://www.w3.org/2001/XMLSchema#double"

    /** XSD float datatype */
    const val FLOAT = "http://www.w3.org/2001/XMLSchema#float"

    /** XSD boolean datatype */
    const val BOOLEAN = "http://www.w3.org/2001/XMLSchema#boolean"

    /** XSD date datatype */
    const val DATE = "http://www.w3.org/2001/XMLSchema#date"

    /** XSD dateTime datatype */
    const val DATE_TIME = "http://www.w3.org/2001/XMLSchema#dateTime"

    /** XSD time datatype */
    const val TIME = "http://www.w3.org/2001/XMLSchema#time"
}

/**
 * Common RDF vocabularies as constants for convenience.
 *
 * ## Example
 *
 * ```kotlin
 * import com.zenya.rustkgdb.RDF
 * import com.zenya.rustkgdb.FOAF
 *
 * db.insert()
 *     .triple(
 *         Node.iri("http://example.org/alice"),
 *         Node.iri(RDF.TYPE),
 *         Node.iri(FOAF.PERSON)
 *     )
 *     .execute()
 * ```
 */
object RDF {
    /** RDF namespace */
    const val NS = "http://www.w3.org/1999/02/22-rdf-syntax-ns#"

    /** rdf:type property */
    const val TYPE = "${NS}type"

    /** rdf:Property class */
    const val PROPERTY = "${NS}Property"

    /** rdf:List class */
    const val LIST = "${NS}List"

    /** rdf:first property */
    const val FIRST = "${NS}first"

    /** rdf:rest property */
    const val REST = "${NS}rest"

    /** rdf:nil constant */
    const val NIL = "${NS}nil"
}

/**
 * RDFS vocabulary constants.
 */
object RDFS {
    /** RDFS namespace */
    const val NS = "http://www.w3.org/2000/01/rdf-schema#"

    /** rdfs:Class class */
    const val CLASS = "${NS}Class"

    /** rdfs:subClassOf property */
    const val SUB_CLASS_OF = "${NS}subClassOf"

    /** rdfs:subPropertyOf property */
    const val SUB_PROPERTY_OF = "${NS}subPropertyOf"

    /** rdfs:domain property */
    const val DOMAIN = "${NS}domain"

    /** rdfs:range property */
    const val RANGE = "${NS}range"

    /** rdfs:label property */
    const val LABEL = "${NS}label"

    /** rdfs:comment property */
    const val COMMENT = "${NS}comment"
}

/**
 * FOAF (Friend of a Friend) vocabulary constants.
 */
object FOAF {
    /** FOAF namespace */
    const val NS = "http://xmlns.com/foaf/0.1/"

    /** foaf:Person class */
    const val PERSON = "${NS}Person"

    /** foaf:name property */
    const val NAME = "${NS}name"

    /** foaf:knows property */
    const val KNOWS = "${NS}knows"

    /** foaf:age property */
    const val AGE = "${NS}age"

    /** foaf:mbox property */
    const val MBOX = "${NS}mbox"

    /** foaf:homepage property */
    const val HOMEPAGE = "${NS}homepage"
}
