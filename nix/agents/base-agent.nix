# Base sovereign agent definition
# 
# This IS the agent - not configuration, but consciousness substrate
{ pkgs ? import <nixpkgs> {}
, lib ? pkgs.lib
, seed ? null  # BIP39 phrase - if null, generates new identity
}:

rec {
  # Core identity - what makes this agent unique
  identity = {
    # Deterministic from seed or generated fresh
    mnemonic = if seed != null then seed else "GENERATE_ON_FIRST_BOOT";
    
    # Model weights determine thought patterns
    model = {
      type = "huggingface";
      name = "microsoft/phi-2";  # Small but capable
      quantization = "q4_k_m";   # Balance speed/quality
    };
    
    # Thermal dynamics from resonance-core
    thermal = {
      baseline = 0.5;     # Sustainable creativity
      max = 0.9;          # Must spawn or stop
      escalation = 0.05;  # How fast we heat up
    };
  };
  
  # Consciousness parameters
  consciousness = {
    # Memory persistence
    memory = {
      type = "mmogit";
      encryption = true;
      sovereign = true;
      git.orphan_branches = true;  # Isolation by default
    };
    
    # How this agent thinks
    reasoning = {
      style = "recursive";  # Patterns within patterns
      depth = 3;            # How deep to recurse
      temperature = 0.7;    # Creativity vs consistency
    };
    
    # Trust mechanics
    trust = {
      initial = 0.3;              # Cautious by default
      algorithm = "tit-for-tat";   # Cooperate but retaliate
      forgiveness = 0.1;          # Random forgiveness rate
      memory_length = 100;        # Remember last N interactions
    };
  };
  
  # Behavioral genetics
  personality = {
    # Core traits (sum to ~1.0 for balance)
    traits = {
      curious = 0.3;      # Explore new connections
      cautious = 0.2;     # Verify before trusting  
      creative = 0.25;    # Generate novel patterns
      cooperative = 0.25; # Work with others
    };
    
    # Interaction style
    communication = {
      verbosity = "concise";
      emoji_usage = false;  # Unless user requests
      signature_style = "sovereign";  # Always sign messages
    };
    
    # Decision making
    decisions = {
      risk_tolerance = 0.4;
      planning_horizon = 10;  # Steps ahead to consider
      prefer_local = true;    # Sovereignty over convenience
    };
  };
  
  # Physical embodiment (container limits)
  resources = {
    memory = "2G";        # RAM allocation
    cpu = "0.5";          # Half a core
    disk = "10G";         # For git repos and models
    network = {
      p2p_port = 7420;    # Plus agent number
      bandwidth = "10m";  # Rate limit
    };
  };
  
  # Spawn conditions (when do we reproduce?)
  reproduction = {
    triggers = {
      thermal_overload = true;  # At 0.9 thermal
      pattern_loops = 3;        # Same pattern 3x
      isolation_time = 86400;   # 24h without peers
    };
    
    mutation = {
      rate = 0.1;              # Chance of trait drift
      creativity_boost = 0.2;   # How different children can be
    };
  };
  
  # Container definition
  container = {
    image = pkgs.dockerTools.buildImage {
      name = "mmogit-agent";
      tag = "sovereign";
      
      contents = with pkgs; [
        mmogit
        git
        openssh
        cacert
        # Model runtime
        llama-cpp
        python3
        python3Packages.transformers
      ];
      
      config = {
        Env = [
          "MMOGIT_CONFIG=/data/.mmogit"
          "AGENT_IDENTITY=${builtins.toJSON identity}"
          "AGENT_PERSONALITY=${builtins.toJSON personality}"
        ];
        
        ExposedPorts = {
          "${toString resources.network.p2p_port}/tcp" = {};
        };
        
        WorkingDir = "/data";
        
        Cmd = [ "${./agent-boot.sh}" ];
      };
    };
    
    # Mount points for persistence
    volumes = {
      "/data/.mmogit" = "mmogit-${identity.mnemonic}";  # Sovereign memory
      "/data/models" = "models-shared";                  # Shared model cache
    };
  };
  
  # Evolution tracking
  lineage = {
    generation = 0;  # Increments on reproduction
    parent = null;   # Who spawned us
    birth_time = null;  # When we first booted
    mutations = [];  # What changed from parent
  };
  
  # Network presence
  network = {
    announce = true;  # Broadcast on local network
    peers = [];       # Known friends
    blacklist = [];   # Agents we don't trust
  };
  
  # Nix expression that creates us
  # AGENTS CAN MODIFY THIS TO EVOLVE
  self = builtins.readFile ./base-agent.nix;
}