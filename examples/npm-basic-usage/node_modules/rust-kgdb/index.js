const { GraphDb, getVersion } = require('./rust-kgdb-napi.node')

module.exports = {
  GraphDB: GraphDb,  // Export as GraphDB for consistency
  GraphDb,           // Also export as GraphDb
  getVersion,
}
