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
    df = pbp.get_game_play_by_play_data(season, game_code)
    return df

def convert_to_ttl(df, output_path: Path, season: int, game_code: int):
    """Convert DataFrame to N-Triples format (full URIs for reliable parsing)."""

    # Use N-Triples format with full URIs
    RDF = "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    RDFS = "http://www.w3.org/2000/01/rdf-schema#"
    OWL = "http://www.w3.org/2002/07/owl#"
    XSD = "http://www.w3.org/2001/XMLSchema#"
    EURO = "http://euroleague.net/ontology#"
    TEAM = "http://euroleague.net/team/"
    PLAYER = "http://euroleague.net/player/"
    EVENT = "http://euroleague.net/event/"
    GAME = "http://euroleague.net/game/"

    lines = []

    # Ontology Classes - All event types MUST be declared as owl:Class
    # for proper schema extraction by HyperMindAgent
    lines.append(f'<{EURO}Game> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Team> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Player> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Event> <{RDF}type> <{OWL}Class> .')
    # Event subclasses - MUST declare as owl:Class AND rdfs:subClassOf
    lines.append(f'<{EURO}Shot> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Shot> <{RDFS}subClassOf> <{EURO}Event> .')
    lines.append(f'<{EURO}Assist> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Assist> <{RDFS}subClassOf> <{EURO}Event> .')
    lines.append(f'<{EURO}Rebound> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Rebound> <{RDFS}subClassOf> <{EURO}Event> .')
    lines.append(f'<{EURO}Turnover> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Turnover> <{RDFS}subClassOf> <{EURO}Event> .')
    lines.append(f'<{EURO}Foul> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Foul> <{RDFS}subClassOf> <{EURO}Event> .')
    lines.append(f'<{EURO}Block> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Block> <{RDFS}subClassOf> <{EURO}Event> .')
    lines.append(f'<{EURO}Steal> <{RDF}type> <{OWL}Class> .')
    lines.append(f'<{EURO}Steal> <{RDFS}subClassOf> <{EURO}Event> .')

    # OWL Properties
    lines.append(f'<{EURO}teammateOf> <{RDF}type> <{OWL}SymmetricProperty> .')
    lines.append(f'<{EURO}assistedBy> <{RDF}type> <{OWL}TransitiveProperty> .')

    ttl = '\n'.join(lines) + '\n'

    # Track unique entities
    teams = set()
    players = {}  # player_id -> team_id
    events = []

    # Game ID
    game_id = f"{season}_{game_code}"
    lines.append(f'<{GAME}{game_id}> <{RDF}type> <{EURO}Game> .')
    lines.append(f'<{GAME}{game_id}> <{RDFS}label> "Euroleague Game {season}/{game_code}" .')

    # Process each play-by-play event
    for idx, row in df.iterrows():
        # Extract fields (column names may vary)
        team = str(row.get('CODETEAM', row.get('team', 'UNKNOWN'))).lower().replace(' ', '_')
        player = str(row.get('PLAYER', row.get('player', 'unknown'))).lower().replace(' ', '_').replace('.', '').replace(',', '_')
        play_type = str(row.get('PLAYTYPE', row.get('playtype', 'event'))).lower().replace(' ', '_')
        description = str(row.get('PLAYINFO', row.get('playinfo', ''))).replace('"', "'")
        quarter = row.get('QUARTER', row.get('quarter', 1))

        # Track teams
        if team and team != 'nan':
            teams.add(team)

        # Track players with their team
        if player and player != 'nan' and team and team != 'nan':
            players[player] = team

        # Create event
        event_id = f"e{idx:05d}"
        # Pass both play_type and description for better type detection
        event_type = map_play_type(play_type, description)

        events.append({
            'id': event_id,
            'type': event_type,
            'player': player,
            'team': team,
            'description': description[:50] if description else event_type,
            'quarter': quarter
        })

    # Add teams
    for team in teams:
        lines.append(f'<{TEAM}{team}> <{RDF}type> <{EURO}Team> .')
        lines.append(f'<{TEAM}{team}> <{RDFS}label> "{team.replace("_", " ").title()}" .')

    # Add players with team relationships
    for player, team in players.items():
        lines.append(f'<{PLAYER}{player}> <{RDF}type> <{EURO}Player> .')
        lines.append(f'<{PLAYER}{player}> <{RDFS}label> "{player.replace("_", " ").title()}" .')
        lines.append(f'<{PLAYER}{player}> <{EURO}playsFor> <{TEAM}{team}> .')

    # Add teammate relationships (players on same team)
    team_players = {}
    for player, team in players.items():
        if team not in team_players:
            team_players[team] = []
        team_players[team].append(player)

    for team, player_list in team_players.items():
        for i, p1 in enumerate(player_list):
            for p2 in player_list[i+1:]:
                lines.append(f'<{PLAYER}{p1}> <{EURO}teammateOf> <{PLAYER}{p2}> .')

    # Add events
    for event in events[:100]:  # Limit to first 100 events for demo
        if event['player'] and event['player'] != 'nan':
            lines.append(f'<{EVENT}{event["id"]}> <{RDF}type> <{EURO}{event["type"]}> .')
            lines.append(f'<{EVENT}{event["id"]}> <{RDFS}label> "{event["description"]}" .')
            lines.append(f'<{EVENT}{event["id"]}> <{EURO}player> <{PLAYER}{event["player"]}> .')
            lines.append(f'<{EVENT}{event["id"]}> <{EURO}team> <{TEAM}{event["team"]}> .')

    ttl = '\n'.join(lines)

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

def map_play_type(play_type: str, description: str = '') -> str:
    """Map play type to ontology class.

    Checks both PLAYTYPE and PLAYINFO (description) for event type keywords.
    This ensures events like "Steal (1)" in description get properly typed.
    """
    mapping = {
        '2pt': 'Shot',
        '3pt': 'Shot',
        'ft': 'Shot',
        'fta': 'Shot',
        'ftm': 'Shot',
        'two pointer': 'Shot',
        'three pointer': 'Shot',
        'free throw': 'Shot',
        'as': 'Assist',
        'ast': 'Assist',
        'assist': 'Assist',
        'reb': 'Rebound',
        'dreb': 'Rebound',
        'oreb': 'Rebound',
        'rebound': 'Rebound',
        'def rebound': 'Rebound',
        'off rebound': 'Rebound',
        'to': 'Turnover',
        'turnover': 'Turnover',
        'pf': 'Foul',
        'foul': 'Foul',
        'blk': 'Block',
        'block': 'Block',
        'stl': 'Steal',
        'steal': 'Steal',
    }

    # First check play_type
    for key, value in mapping.items():
        if key in play_type.lower():
            return value

    # Then check description as fallback
    desc_lower = description.lower()
    for key, value in mapping.items():
        if key in desc_lower:
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
