#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = ["euroleague-api"]
# ///
"""
Euroleague Play-by-Play Data to RDF TTL Converter

Uses the euroleague-api package to fetch game data and converts it to RDF triples.
This is the same data source used in the Medium article:
https://medium.com/@skontopo2009/representing-euroleague-play-by-play-data-as-a-knowledge-graph

Usage (with uv):
    uv run scripts/euroleague-to-ttl.py --season 2024 --game-code 1

Usage (with pip):
    pip install euroleague-api
    python scripts/euroleague-to-ttl.py --season 2024 --game-code 1

Output:
    data/euroleague-game.ttl (can be loaded with db.loadTtl in Node.js)
"""

import argparse
import json
import sys
from pathlib import Path

def check_dependencies():
    """Check if required dependencies are installed."""
    try:
        from euroleague_api.play_by_play_data import PlayByPlay
        return True
    except ImportError:
        print("ERROR: euroleague-api not installed")
        print("Install with: pip install euroleague-api")
        return False

def fetch_game_data(season: int, game_code: int):
    """Fetch play-by-play data for a specific game."""
    from euroleague_api.play_by_play_data import PlayByPlay

    print(f"Fetching game data: Season {season}, Game {game_code}")
    pbp = PlayByPlay()
    df = pbp.get_game_play_by_play(season, game_code)
    return df

def convert_to_ttl(df, output_path: Path, season: int, game_code: int):
    """Convert DataFrame to RDF TTL format."""

    # TTL prefixes
    ttl = """@prefix euro: <http://euroleague.net/ontology#> .
@prefix team: <http://euroleague.net/team/> .
@prefix player: <http://euroleague.net/player/> .
@prefix event: <http://euroleague.net/event/> .
@prefix game: <http://euroleague.net/game/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Ontology Classes
euro:Game rdf:type owl:Class .
euro:Team rdf:type owl:Class .
euro:Player rdf:type owl:Class .
euro:Event rdf:type owl:Class .
euro:Shot rdfs:subClassOf euro:Event .
euro:Assist rdfs:subClassOf euro:Event .
euro:Rebound rdfs:subClassOf euro:Event .
euro:Turnover rdfs:subClassOf euro:Event .
euro:Foul rdfs:subClassOf euro:Event .
euro:Block rdfs:subClassOf euro:Event .
euro:Steal rdfs:subClassOf euro:Event .

# OWL Properties for Reasoning
euro:teammateOf rdf:type owl:SymmetricProperty ;
    rdfs:label "teammate relationship (symmetric)" .
euro:assistedBy rdf:type owl:TransitiveProperty ;
    rdfs:label "assisted by (transitive chain)" .

"""

    # Track unique entities
    teams = set()
    players = {}  # player_id -> team_id
    events = []

    # Game ID
    game_id = f"{season}_{game_code}"
    ttl += f"""
# Game Instance
game:{game_id} rdf:type euro:Game ;
    rdfs:label "Euroleague Game {season}/{game_code}" ;
    euro:season "{season}"^^xsd:integer .

"""

    # Process each play-by-play event
    for idx, row in df.iterrows():
        # Extract fields (column names may vary)
        team = str(row.get('CODETEAM', row.get('team', 'UNKNOWN'))).lower().replace(' ', '_')
        player = str(row.get('PLAYER', row.get('player', 'unknown'))).lower().replace(' ', '_').replace('.', '')
        play_type = str(row.get('PLAYTYPE', row.get('playtype', 'event'))).lower().replace(' ', '_')
        description = str(row.get('PLAYINFO', row.get('playinfo', '')))
        quarter = row.get('QUARTER', row.get('quarter', 1))

        # Track teams
        if team and team != 'nan':
            teams.add(team)

        # Track players with their team
        if player and player != 'nan' and team and team != 'nan':
            players[player] = team

        # Create event
        event_id = f"e{idx:05d}"
        event_type = map_play_type(play_type)

        events.append({
            'id': event_id,
            'type': event_type,
            'player': player,
            'team': team,
            'description': description,
            'quarter': quarter
        })

    # Add teams
    for team in teams:
        ttl += f"""
team:{team} rdf:type euro:Team ;
    rdfs:label "{team.replace('_', ' ').title()}" .
"""

    # Add players with team relationships
    for player, team in players.items():
        ttl += f"""
player:{player} rdf:type euro:Player ;
    rdfs:label "{player.replace('_', ' ').title()}" ;
    euro:playsFor team:{team} .
"""

    # Add teammate relationships (players on same team)
    team_players = {}
    for player, team in players.items():
        if team not in team_players:
            team_players[team] = []
        team_players[team].append(player)

    for team, player_list in team_players.items():
        for i, p1 in enumerate(player_list):
            for p2 in player_list[i+1:]:
                ttl += f"player:{p1} euro:teammateOf player:{p2} .\n"

    # Add events
    for event in events[:100]:  # Limit to first 100 events for demo
        if event['player'] and event['player'] != 'nan':
            ttl += f"""
event:{event['id']} rdf:type euro:{event['type']} ;
    rdfs:label "{event['description'][:50] if event['description'] else event['type']}" ;
    euro:player player:{event['player']} ;
    euro:team team:{event['team']} ;
    euro:quarter "{event['quarter']}"^^xsd:integer .
"""

    # Write output
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(ttl)

    print(f"\nGenerated TTL file: {output_path}")
    print(f"  Teams: {len(teams)}")
    print(f"  Players: {len(players)}")
    print(f"  Events: {min(len(events), 100)}")
    print(f"\nLoad in Node.js:")
    print(f"  const fs = require('fs')")
    print(f"  const ttl = fs.readFileSync('{output_path}', 'utf-8')")
    print(f"  db.loadTtl(ttl, null)")

def map_play_type(play_type: str) -> str:
    """Map play type to ontology class."""
    mapping = {
        '2pt': 'Shot',
        '3pt': 'Shot',
        'ft': 'Shot',
        'fta': 'Shot',
        'ftm': 'Shot',
        'as': 'Assist',
        'ast': 'Assist',
        'assist': 'Assist',
        'reb': 'Rebound',
        'dreb': 'Rebound',
        'oreb': 'Rebound',
        'to': 'Turnover',
        'turnover': 'Turnover',
        'pf': 'Foul',
        'foul': 'Foul',
        'blk': 'Block',
        'block': 'Block',
        'stl': 'Steal',
        'steal': 'Steal',
    }

    for key, value in mapping.items():
        if key in play_type.lower():
            return value
    return 'Event'

def main():
    parser = argparse.ArgumentParser(
        description='Convert Euroleague play-by-play data to RDF TTL format'
    )
    parser.add_argument('--season', type=int, default=2024,
                        help='Season year (default: 2024)')
    parser.add_argument('--game-code', type=int, default=1,
                        help='Game code (default: 1)')
    parser.add_argument('--output', type=str, default='data/euroleague-game.ttl',
                        help='Output TTL file path')

    args = parser.parse_args()

    if not check_dependencies():
        sys.exit(1)

    try:
        df = fetch_game_data(args.season, args.game_code)
        convert_to_ttl(df, Path(args.output), args.season, args.game_code)
    except Exception as e:
        print(f"Error: {e}")
        print("\nTip: Make sure the season/game-code combination is valid")
        print("Try: python scripts/euroleague-to-ttl.py --season 2023 --game-code 1")
        sys.exit(1)

if __name__ == '__main__':
    main()
