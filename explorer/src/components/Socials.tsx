import React from 'react';
import { Box, IconButton } from '@mui/material';
import TelegramIcon from '@mui/icons-material/Telegram';
import GitHubIcon from '@mui/icons-material/GitHub';
import TwitterIcon from '@mui/icons-material/Twitter';
import { GITHUB_LINK, TELEGRAM_LINK, TWITTER_LINK } from 'src/api/constants';

export const Socials: React.FC<{ disableDarkMode?: boolean }> = ({
  disableDarkMode,
}) => (
  <Box>
    <IconButton component="a" href={TELEGRAM_LINK} target="_blank">
      <TelegramIcon sx={{ color: disableDarkMode ? 'white' : null }} />
    </IconButton>
    <IconButton component="a" href={GITHUB_LINK} target="_blank">
      <GitHubIcon sx={{ color: disableDarkMode ? 'white' : null }} />
    </IconButton>
    <IconButton component="a" href={TWITTER_LINK} target="_blank">
      <TwitterIcon sx={{ color: disableDarkMode ? 'white' : null }} />
    </IconButton>
  </Box>
);

Socials.defaultProps = {
  disableDarkMode: false,
};
