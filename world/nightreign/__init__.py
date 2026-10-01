        self.push_precollected(self.create_item(start))
        # Wylder is the default active body. A locked active body exits on the board.
        self.push_precollected(self.create_item("Nightfarer - Wylder"))
        roster = [n for n in self._nightfarers() if n != "Nightfarer - Wylder"]
        self.random.shuffle(roster)
        start_count = min(int(self.options.starting_nightfarers), len(roster))
        for name in roster[:start_count]:
            self.push_precollected(self.create_item(name))
