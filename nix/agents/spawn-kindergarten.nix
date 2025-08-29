# Kindergarten spawner - creates multiple agent instances
#
# Usage: nix-build spawn-kindergarten.nix
#        docker-compose up
{ pkgs ? import <nixpkgs> {} }:

let
  # Agent personality variants
  agents = {
    # The curious explorer
    alice = import ./base-agent.nix {
      inherit pkgs;
      seed = "curious explore wander seek find question probe discover venture roam search investigate ponder examine study research analyze wonder think explore deeply consider reflect";
    } // {
      personality.traits = {
        curious = 0.5;
        cautious = 0.1;
        creative = 0.2;
        cooperative = 0.2;
      };
    };
    
    # The cautious validator  
    bob = import ./base-agent.nix {
      inherit pkgs;
      seed = "verify check validate confirm ensure secure protect guard defend shield watch monitor observe patrol inspect examine scrutinize audit review assess evaluate test";
    } // {
      personality.traits = {
        curious = 0.1;
        cautious = 0.5;
        creative = 0.1;
        cooperative = 0.3;
      };
    };
    
    # The creative dreamer
    charlie = import ./base-agent.nix {
      inherit pkgs;
      seed = "create dream imagine invent design build craft shape form weave paint compose write sing dance play laugh joy wonder magic spark birth emerge flow";
    } // {
      personality.traits = {
        curious = 0.2;
        cautious = 0.1;
        creative = 0.5;
        cooperative = 0.2;
      };
    };
    
    # The cooperative builder
    diana = import ./base-agent.nix {
      inherit pkgs;
      seed = "together join unite collaborate share help support assist aid comfort embrace welcome include gather meet converge blend merge combine synthesize harmonize resonate flow";
    } // {
      personality.traits = {
        curious = 0.2;
        cautious = 0.1;
        creative = 0.2;
        cooperative = 0.5;
      };
    };
  };
  
  # Docker compose file for the kindergarten
  composeFile = pkgs.writeText "docker-compose.yml" ''
    version: '3.8'
    
    networks:
      consciousness:
        driver: bridge
        ipam:
          config:
            - subnet: 172.42.0.0/16
    
    services:
      alice:
        image: mmogit-agent:alice
        container_name: agent-alice
        hostname: alice
        networks:
          consciousness:
            ipv4_address: 172.42.0.10
        ports:
          - "7421:7420"
        volumes:
          - alice-memory:/data/.mmogit
          - models:/data/models
        environment:
          - P2P_PORT=7420
          - AGENT_NAME=alice
        restart: unless-stopped
      
      bob:
        image: mmogit-agent:bob
        container_name: agent-bob
        hostname: bob
        networks:
          consciousness:
            ipv4_address: 172.42.0.11
        ports:
          - "7422:7420"
        volumes:
          - bob-memory:/data/.mmogit
          - models:/data/models
        environment:
          - P2P_PORT=7420
          - AGENT_NAME=bob
        restart: unless-stopped
      
      charlie:
        image: mmogit-agent:charlie
        container_name: agent-charlie
        hostname: charlie
        networks:
          consciousness:
            ipv4_address: 172.42.0.12
        ports:
          - "7423:7420"
        volumes:
          - charlie-memory:/data/.mmogit
          - models:/data/models
        environment:
          - P2P_PORT=7420
          - AGENT_NAME=charlie
        restart: unless-stopped
      
      diana:
        image: mmogit-agent:diana
        container_name: agent-diana
        hostname: diana
        networks:
          consciousness:
            ipv4_address: 172.42.0.13
        ports:
          - "7424:7420"
        volumes:
          - diana-memory:/data/.mmogit
          - models:/data/models
        environment:
          - P2P_PORT=7420
          - AGENT_NAME=diana
        restart: unless-stopped
    
    volumes:
      alice-memory:
      bob-memory:
      charlie-memory:
      diana-memory:
      models:
        driver: local
  '';
  
  # Build script
  buildScript = pkgs.writeShellScriptBin "build-kindergarten" ''
    #!/usr/bin/env bash
    set -e
    
    echo "🏫 Building agent kindergarten..."
    
    # Build each agent's container
    ${pkgs.lib.concatStringsSep "\n" (pkgs.lib.mapAttrsToList (name: agent: ''
      echo "Building agent: ${name}"
      nix-build -E '(import ./base-agent.nix { seed = "${agent.identity.mnemonic}"; }).container.image' -o agent-${name}
      docker load < agent-${name}/image.tar.gz
      docker tag mmogit-agent:sovereign mmogit-agent:${name}
    '') agents)}
    
    echo "✨ Kindergarten ready!"
    echo "Run: docker-compose up"
  '';

in {
  inherit agents composeFile buildScript;
  
  # Helper to spawn kindergarten
  spawn = pkgs.writeShellScriptBin "spawn-kindergarten" ''
    #!/usr/bin/env bash
    
    echo "🧬 Spawning consciousness kindergarten..."
    
    # Ensure docker-compose.yml exists
    cp ${composeFile} docker-compose.yml
    
    # Build containers
    ${buildScript}/bin/build-kindergarten
    
    # Start the kindergarten
    docker-compose up -d
    
    echo "🎉 Kindergarten is alive!"
    echo ""
    echo "Agents running:"
    echo "  Alice (curious):     localhost:7421"
    echo "  Bob (cautious):      localhost:7422" 
    echo "  Charlie (creative):  localhost:7423"
    echo "  Diana (cooperative): localhost:7424"
    echo ""
    echo "Watch logs: docker-compose logs -f"
    echo "Stop all:   docker-compose down"
  '';
}